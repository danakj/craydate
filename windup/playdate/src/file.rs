use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::NonNull;

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;
use crate::Error;

/// Returns human-readable text describing the most recent file error.
fn last_err(state: &CApiState) -> String {
  let ptr = unsafe { state.cfile.geterr.unwrap()() };
  match unsafe { crate::null_terminated::parse_null_terminated_utf8(ptr) } {
    Ok(s) => s.into(),
    Err(e) => format!(
      "File: unable to parse UTF-8 error string from Playdate. {}",
      e
    ),
  }
}

#[derive(Debug)]
pub struct File {
  pub(crate) state: &'static CApiState,
}
impl File {
  pub(crate) fn new(state: &'static CApiState) -> Self {
    File { state }
  }

  /// Returns an iterator with every file or subfolder found at `path`.
  ///
  /// Subfolders are indicated by a slash '/' suffix in the filename. `list_files()` does not
  /// recurse into subfolders.
  pub fn list_files(&self, path: &str) -> Result<ListFilesIterator, Error> {
    ListFilesIterator::new(self.state, path)
      .or_else(|e| Err(format!("{} (Playdate: {})", e, last_err(self.state)).into()))
  }

  /// Reads information about the file or folder at `path`.
  pub fn stat(&self, path: &str) -> Result<FileStat, Error> {
    let mut s = core::mem::MaybeUninit::<CFileStat>::uninit();
    let result = unsafe {
      self.state.cfile.stat.unwrap()(path.to_null_terminated_utf8().as_ptr(), s.as_mut_ptr())
    };
    match result {
      0 => {
        let s = unsafe { s.assume_init() };
        Ok(FileStat {
          is_folder: s.isdir != 0,
          size: s.size,
          modified: FileTimestamp {
            year: s.m_year,
            month: s.m_month,
            day: s.m_day,
            hour: s.m_hour,
            minute: s.m_minute,
            second: s.m_second,
          },
        })
      }
      _ => Err(
        format!(
          "error reading stat info for '{}' (Playdate: {})",
          path,
          last_err(self.state)
        )
        .into(),
      ),
    }
  }

  //  Creates a folder at the given `path`.
  //
  // This function does not create intermediate folders. The path will be relocated relative to the
  // Data/<gameid> folder.
  pub fn make_folder(&self, path: &str) -> Result<(), Error> {
    let result =
      unsafe { self.state.cfile.mkdir.unwrap()(path.to_null_terminated_utf8().as_ptr()) };
    match result {
      0 => Ok(()),
      _ => Err(
        format!(
          "error making folder '{}' (Playdate: {})",
          path,
          last_err(self.state)
        )
        .into(),
      ),
    }
  }

  /// Renames the file or folder at `from` to `to`.
  ///
  /// This function will overwrite the file at `to` without confirmation, but will fail to rename a
  /// folder when another exists with the same name. It does not create intermediate folders.
  pub fn rename(&self, from: &str, to: &str) -> Result<(), Error> {
    let result = unsafe {
      self.state.cfile.rename.unwrap()(
        from.to_null_terminated_utf8().as_ptr(),
        to.to_null_terminated_utf8().as_ptr(),
      )
    };
    match result {
      0 => Ok(()),
      _ => Err(
        format!(
          "error renaming file/folder '{}' to '{}' (Playdate: {})",
          from,
          to,
          last_err(self.state)
        )
        .into(),
      ),
    }
  }

  /// Read the entire contents of the file at `path`.
  pub fn read_file(&self, path: &str) -> Result<Vec<u8>, Error> {
    // To open a file for reading in the simulator and on the hardware you currently have to set the mode to kFileRead|kFileReadData
    let ptr = NonNull::new(unsafe {
      self.state.cfile.open.unwrap()(
        path.to_null_terminated_utf8().as_ptr(),
        playdate_sys::FileOptions::kFileRead | playdate_sys::FileOptions::kFileReadData,
      )
    });
    match ptr {
      None => Err(
        format!(
          "error opening file '{}' for reading (Playdate: {})",
          path,
          last_err(self.state)
        )
        .into(),
      ),
      Some(handle) => {
        let f = OpenFile::new(self.state, handle);
        let read_result = f.read_file();
        let _close_result = f.close(); // We don't care if close() fails on a read.
        read_result
      }
    }
  }

  /// Write `contents` into the file at `path`.
  ///
  /// If a file exists at `path` it will be overwritten, otherwise a file will be created. If a
  /// folder exists at `path`, the write will fail.
  pub fn write_file(&self, path: &str, contents: &[u8]) -> Result<(), Error> {
    // To open a file for reading in the simulator and on the hardware you currently have to set the mode to kFileRead|kFileReadData
    let ptr = NonNull::new(unsafe {
      self.state.cfile.open.unwrap()(
        path.to_null_terminated_utf8().as_ptr(),
        playdate_sys::FileOptions::kFileWrite,
      )
    });
    match ptr {
      None => Err(
        format!(
          "error opening file '{}' for writing (Playdate: {})",
          path,
          last_err(self.state)
        )
        .into(),
      ),
      Some(handle) => {
        let f = OpenFile::new(self.state, handle);
        let write_result = f.write_file(contents);
        // If close() fails on a write, we return an error as the file content may not be complete.
        f.close()?;
        write_result
      }
    }
  }

  /// Deletes the file or folder at `path`.
  ///
  /// TODO: Currently the simulator appears to always fail with "Permission denied".
  ///
  /// If the path is to a non-empty folder, it will fail. The path will be relocated relative to the
  /// Data/<gameid> folder, so it can not refer to things that are part of the game's pdx image.
  pub fn delete(&self, path: &str) -> Result<(), Error> {
    let result =
      unsafe { self.state.cfile.unlink.unwrap()(path.to_null_terminated_utf8().as_ptr(), 0) };
    match result {
      0 => Ok(()),
      _ => Err(
        format!(
          "error deleting file/folder '{}' (Playdate: {})",
          path,
          last_err(self.state)
        )
        .into(),
      ),
    }
  }

  /// Deletes the file at path, or the folder and its contents.
  ///
  /// TODO: Currently the simulator appears to always fail with "Permission denied".
  ///
  /// If the path is a folder, and all files and folders inside it are deleted as well. The path
  /// will be relocated relative to the Data/<gameid> folder, so it can not refer to things that are
  /// part of the game's pdx image.
  pub fn delete_recursive(&self, path: &str) -> Result<(), Error> {
    let result =
      unsafe { self.state.cfile.unlink.unwrap()(path.to_null_terminated_utf8().as_ptr(), 1) };
    match result {
      0 => Ok(()),
      _ => Err(
        format!(
          "error recursively deleting file/folder '{}' (Playdate: {})",
          path,
          last_err(self.state)
        )
        .into(),
      ),
    }
  }
}

/// An open file which can be read from and written to.
///
/// The close() function _must_ be called in order to destroy the `OpenFile` object. Dropping the
/// OpenFile without calling close() will panic/abort.
#[derive(Debug)]
struct OpenFile {
  state: &'static CApiState,
  handle: NonNull<COpenFile>,
  closed: bool,
}
impl OpenFile {
  fn new(state: &'static CApiState, handle: NonNull<COpenFile>) -> Self {
    OpenFile {
      state,
      handle,
      closed: false,
    }
  }

  /// Read the entire contents of the file.
  pub fn read_file(&self) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    const BUF_SIZE: u32 = 256;
    let mut buf = [0; BUF_SIZE as usize];
    loop {
      let result = unsafe {
        self.state.cfile.read.unwrap()(
          self.handle.as_ptr(),
          buf.as_mut_ptr() as *mut c_void,
          BUF_SIZE,
        )
      };
      let bytes = match result {
        // Reached the end of the file.
        0 => break,
        // Return immediately on an error.
        -1 => Err(format!(
          "error reading from file (Playdate: {}",
          last_err(self.state)
        ))?,
        num_read_bytes => num_read_bytes as usize,
      };
      out.extend(buf[0..bytes].into_iter());
    }
    Ok(out)
  }

  /// Write the entire contents of the file.
  pub fn write_file(&self, contents: &[u8]) -> Result<(), Error> {
    // TODO: This would be needed if we support other operations beyond read/write the whole
    // file.
    // self.state.cfile.seek.unwrap()(self.handle.as_ptr(), 0, playdate_sys::SEEK_SET as i32);

    const BUF_SIZE: usize = 256;
    for buf in contents.chunks(BUF_SIZE) {
      let mut written_from_buffer = 0;
      loop {
        let result = unsafe {
          self.state.cfile.write.unwrap()(
            self.handle.as_ptr().add(written_from_buffer),
            buf.as_ptr() as *const c_void,
            (buf.len() - written_from_buffer) as u32,
          )
        };
        written_from_buffer += match result {
          // Return immediately on an error.
          -1 => Err(format!(
            "error writing to file (Playdate: {}",
            last_err(self.state)
          ))?,
          num_written_bytes => num_written_bytes as usize,
        };
        if written_from_buffer == buf.len() {
          break;
        }
      }
    }
    Ok(())
  }

  /// Close the file. This function _must_ be called in order to destroy the `OpenFile` object.
  ///
  /// Dropping the OpenFile without calling close() will panic/abort.
  pub fn close(mut self) -> Result<(), Error> {
    self.closed = true;
    let result = unsafe { self.state.cfile.close.unwrap()(self.handle.as_ptr()) };
    match result {
      0 => Ok(()),
      _ => Err(format!("error closing file (Playdate: {})", last_err(self.state)).into()),
    }
  }
}
impl Drop for OpenFile {
  fn drop(&mut self) {
    if !self.closed {
      crate::debug::log("ERROR: OpenFile dropped without calling close()");
      assert!(self.closed);
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileTimestamp {
  pub year: i32,
  pub month: i32,
  pub day: i32,
  pub hour: i32,
  pub minute: i32,
  pub second: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileStat {
  pub is_folder: bool,
  pub size: u32,
  pub modified: FileTimestamp,
}

#[derive(Debug)]
pub struct ListFilesIterator {
  iter: alloc::vec::IntoIter<String>,
}

impl ListFilesIterator {
  fn new(state: &'static CApiState, path: &str) -> Result<Self, String> {
    let mut v = Vec::<String>::new();
    unsafe extern "C" fn add_file(filename: *const u8, userdata: *mut c_void) {
      let v = &mut *(userdata as *mut Vec<String>);
      v.push(crate::null_terminated::parse_null_terminated_utf8(filename).unwrap().into());
    }
    let result = unsafe {
      state.cfile.listfiles.unwrap()(
        path.to_null_terminated_utf8().as_ptr(),
        Some(add_file),
        &mut v as *mut Vec<String> as *mut c_void,
      )
    };
    if result == 0 {
      Ok(ListFilesIterator {
        iter: v.into_iter(),
      })
    } else {
      Err(format!(
        "no folder exists at '{}', or it can't be opened",
        path
      ))
    }
  }
}

impl Iterator for ListFilesIterator {
  type Item = String;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}
