use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::NonNull;

use super::file_stat::FileStat;
use super::file_timestamp::FileTimestamp;
use super::open_file::OpenFile;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;
use crate::Error;

/// Returns human-readable text describing the most recent file error.
pub(super) fn last_err() -> String {
  let ptr = unsafe { File::fns().geterr.unwrap()() };
  match unsafe { crate::null_terminated::parse_null_terminated_utf8(ptr) } {
    Ok(s) => s.into(),
    Err(e) => format!(
      "File: unable to parse UTF-8 error string from Playdate. {}",
      e
    ),
  }
}

#[derive(Debug)]
pub struct File;
impl File {
  pub(crate) fn new() -> Self {
    File
  }

  /// Returns an iterator with every file or subfolder found at `path`.
  ///
  /// Subfolders are indicated by a slash '/' suffix in the filename. `list_files()` does not
  /// recurse into subfolders.
  pub fn list_files(&self, path: &str) -> Result<ListFilesIterator, Error> {
    ListFilesIterator::new(path)
      .or_else(|e| Err(format!("{} (Playdate: {})", e, last_err()).into()))
  }

  /// Reads information about the filemod or folder at `path`.
  pub fn stat(&self, path: &str) -> Result<FileStat, Error> {
    let mut s = core::mem::MaybeUninit::<CFileStat>::uninit();
    let result =
      unsafe { Self::fns().stat.unwrap()(path.to_null_terminated_utf8().as_ptr(), s.as_mut_ptr()) };
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
          last_err()
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
    let result = unsafe { Self::fns().mkdir.unwrap()(path.to_null_terminated_utf8().as_ptr()) };
    match result {
      0 => Ok(()),
      _ => Err(format!("error making folder '{}' (Playdate: {})", path, last_err()).into()),
    }
  }

  /// Renames the file or folder at `from` to `to`.
  ///
  /// This function will overwrite the file at `to` without confirmation, but will fail to rename a
  /// folder when another exists with the same name. It does not create intermediate folders.
  pub fn rename(&self, from: &str, to: &str) -> Result<(), Error> {
    let result = unsafe {
      Self::fns().rename.unwrap()(
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
          last_err()
        )
        .into(),
      ),
    }
  }

  /// Read the entire contents of the file at `path`.
  pub fn read_file(&self, path: &str) -> Result<Vec<u8>, Error> {
    // To open a file for reading in the simulator and on the hardware you currently have to set the mode to kFileRead|kFileReadData
    let ptr = NonNull::new(unsafe {
      Self::fns().open.unwrap()(
        path.to_null_terminated_utf8().as_ptr(),
        playdate_sys::FileOptions::kFileRead | playdate_sys::FileOptions::kFileReadData,
      )
    });
    match ptr {
      None => Err(
        format!(
          "error opening file '{}' for reading (Playdate: {})",
          path,
          last_err()
        )
        .into(),
      ),
      Some(handle) => {
        let f = OpenFile::new(handle);
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
      Self::fns().open.unwrap()(
        path.to_null_terminated_utf8().as_ptr(),
        playdate_sys::FileOptions::kFileWrite,
      )
    });
    match ptr {
      None => Err(
        format!(
          "error opening file '{}' for writing (Playdate: {})",
          path,
          last_err()
        )
        .into(),
      ),
      Some(handle) => {
        let f = OpenFile::new(handle);
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
    let result = unsafe { Self::fns().unlink.unwrap()(path.to_null_terminated_utf8().as_ptr(), 0) };
    match result {
      0 => Ok(()),
      _ => Err(
        format!(
          "error deleting file/folder '{}' (Playdate: {})",
          path,
          last_err()
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
    let result = unsafe { Self::fns().unlink.unwrap()(path.to_null_terminated_utf8().as_ptr(), 1) };
    match result {
      0 => Ok(()),
      _ => Err(
        format!(
          "error recursively deleting file/folder '{}' (Playdate: {})",
          path,
          last_err()
        )
        .into(),
      ),
    }
  }

  pub(crate) fn fns() -> &'static playdate_sys::playdate_file {
    CApiState::get().cfile
  }
}

#[derive(Debug)]
pub struct ListFilesIterator {
  iter: alloc::vec::IntoIter<String>,
}

impl ListFilesIterator {
  fn new(path: &str) -> Result<Self, String> {
    let mut v = Vec::<String>::new();
    unsafe extern "C" fn add_file(filename: *const u8, userdata: *mut c_void) {
      let v = &mut *(userdata as *mut Vec<String>);
      v.push(crate::null_terminated::parse_null_terminated_utf8(filename).unwrap().into());
    }
    let result = unsafe {
      File::fns().listfiles.unwrap()(
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
