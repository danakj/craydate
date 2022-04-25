use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::NonNull;

use super::file_path_stat::FilePathStat;
use super::file_path_timestamp::FilePathTimestamp;
use super::open_file::OpenFile;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;
use crate::{FilePathError, RenameFilePathError};

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

/// Access to the file system of the Playdate device.
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
  pub fn list_files(&self, path: &str) -> Result<impl Iterator<Item = String>, FilePathError> {
    ListFilesIterator::new(path).ok_or_else(|| FilePathError {
      path: String::from(path),
      playdate: last_err(),
    })
  }

  /// Reads information about the filemod or folder at `path`.
  pub fn stat(&self, path: &str) -> Result<FilePathStat, FilePathError> {
    let mut s = core::mem::MaybeUninit::<CFileStat>::uninit();
    let result =
      unsafe { Self::fns().stat.unwrap()(path.to_null_terminated_utf8().as_ptr(), s.as_mut_ptr()) };
    match result {
      0 => {
        let s = unsafe { s.assume_init() };
        let modified = FilePathTimestamp {
          year: s.m_year,
          month: s.m_month,
          day: s.m_day,
          hour: s.m_hour,
          minute: s.m_minute,
          second: s.m_second,
        };
        if s.isdir != 0 {
          Ok(FilePathStat::Folder { modified })
        } else {
          Ok(FilePathStat::File {
            size: s.size,
            modified,
          })
        }
      }
      _ => Err(FilePathError {
        path: String::from(path),
        playdate: last_err(),
      }),
    }
  }

  // Creates a folder at the given `path`.
  //
  // This function does not create intermediate folders. The path will be relocated relative to the
  // Data/<gameid> folder.
  pub fn make_folder(&self, path: &str) -> Result<(), FilePathError> {
    let result = unsafe { Self::fns().mkdir.unwrap()(path.to_null_terminated_utf8().as_ptr()) };
    match result {
      0 => Ok(()),
      _ => Err(FilePathError {
        path: String::from(path),
        playdate: last_err(),
      }),
    }
  }

  /// Renames the file or folder at `from` to `to`.
  ///
  /// This function will overwrite the file at `to` without confirmation, but will fail to rename a
  /// folder when another exists with the same name. It does not create intermediate folders.
  pub fn rename(&self, from: &str, to: &str) -> Result<(), RenameFilePathError> {
    let result = unsafe {
      Self::fns().rename.unwrap()(
        from.to_null_terminated_utf8().as_ptr(),
        to.to_null_terminated_utf8().as_ptr(),
      )
    };
    match result {
      0 => Ok(()),
      _ => Err(RenameFilePathError {
        from_path: String::from(from),
        to_path: String::from(to),
        playdate: last_err(),
      }),
    }
  }

  /// Read the entire contents of the file at `path`.
  ///
  /// The function will try to read from the game's data folder, and if it cannot find the file
  /// there, it will fallback to look in the game pdx.
  pub fn read_file(&self, path: &str) -> Result<Vec<u8>, FilePathError> {
    // To open a file for reading in the simulator and on the hardware you currently have to set the mode to kFileRead|kFileReadData
    let ptr = NonNull::new(unsafe {
      Self::fns().open.unwrap()(
        path.to_null_terminated_utf8().as_ptr(),
        // TODO: Do we need to give the user the option to exclude the data folder or pdx?
        playdate_sys::FileOptions::kFileReadData | playdate_sys::FileOptions::kFileRead,
      )
    });
    match ptr {
      None => Err(FilePathError {
        path: String::from(path),
        playdate: last_err(),
      }),
      Some(handle) => {
        let mut f = OpenFile::new(handle);
        let read_result = f.read_file();
        let _close_result = f.close(); // We don't care if close() fails on a read.
        read_result.ok_or_else(|| FilePathError {
          path: String::from(path),
          playdate: last_err(),
        })
      }
    }
  }

  /// Write `contents` into the file at `path` in the game's data folder.
  ///
  /// If a file exists at `path` it will be overwritten, otherwise a file will be created. If a
  /// folder exists at `path`, the write will fail.
  pub fn write_file(&self, path: &str, contents: &[u8]) -> Result<(), FilePathError> {
    // To open a file for reading in the simulator and on the hardware you currently have to set the mode to kFileRead|kFileReadData
    let ptr = NonNull::new(unsafe {
      Self::fns().open.unwrap()(
        path.to_null_terminated_utf8().as_ptr(),
        playdate_sys::FileOptions::kFileWrite,
      )
    });
    match ptr {
      None => Err(FilePathError {
        path: String::from(path),
        playdate: last_err(),
      }),
      Some(handle) => {
        let mut f = OpenFile::new(handle);
        let write_result = f.write_file(contents);
        // If close() fails on a write, we return an error as the file content may not be complete.
        if f.close() && write_result {
          Ok(())
        } else {
          Err(FilePathError {
            path: String::from(path),
            playdate: last_err(),
          })
        }
      }
    }
  }

  /// Deletes the file or folder at `path` in the game's data folder.
  ///
  /// BUG: This is currently broken, and always reports "permission denied" in the simulator:
  /// <https://devforum.play.date/t/unlink-gives-permission-denied-in-c-api-in-windows-simulator/4979>
  ///
  /// If the path is to a non-empty folder, it will fail. The path will be relocated relative to the
  /// Data/<gameid> folder, so it can not refer to things that are part of the game's pdx image.
  pub fn delete(&self, path: &str) -> Result<(), FilePathError> {
    let result =
      unsafe { Self::fns().unlink.unwrap()(path.to_null_terminated_utf8().as_ptr(), false as i32) };
    match result {
      0 => Ok(()),
      _ => Err(FilePathError {
        path: String::from(path),
        playdate: last_err(),
      }),
    }
  }

  /// Deletes the file at path, or the folder and its contents. The path is searched for in the
  /// game's data folder.
  ///
  /// BUG: This is currently broken, and always reports "permission denied" in the simulator:
  /// <https://devforum.play.date/t/unlink-gives-permission-denied-in-c-api-in-windows-simulator/4979>
  ///
  /// If the path is a folder, and all files and folders inside it are deleted as well. The path
  /// will be relocated relative to the Data/<gameid> folder, so it can not refer to things that are
  /// part of the game's pdx image.
  pub fn delete_recursive(&self, path: &str) -> Result<(), FilePathError> {
    let result =
      unsafe { Self::fns().unlink.unwrap()(path.to_null_terminated_utf8().as_ptr(), true as i32) };
    match result {
      0 => Ok(()),
      _ => Err(FilePathError {
        path: String::from(path),
        playdate: last_err(),
      }),
    }
  }

  pub(crate) fn fns() -> &'static playdate_sys::playdate_file {
    CApiState::get().cfile
  }
}

#[derive(Debug)]
pub struct ListFilesIterator;
impl ListFilesIterator {
  fn new(path: &str) -> Option<alloc::vec::IntoIter<String>> {
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
      Some(v.into_iter())
    } else {
      None
    }
  }
}
