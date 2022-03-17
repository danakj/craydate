use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;

use crate::capi_state::CApiState;
use crate::null_terminated::ToNullTerminatedString;
use crate::Error;

#[derive(Debug)]
pub struct File {
  pub(crate) state: &'static CApiState,
}
impl File {
  pub(crate) fn new(state: &'static CApiState) -> Self {
    File { state }
  }

  /// Returns human-readable text describing the most recent file error.
  fn last_err(&self) -> String {
    let ptr = unsafe { self.state.cfile.geterr.unwrap()() };
    match unsafe { crate::null_terminated::parse_null_terminated_utf8(ptr) } {
      Ok(s) => s.into(),
      Err(e) => format!(
        "File: unable to parse UTF-8 error string from Playdate. {}",
        e
      ),
    }
  }

  /// Returns an iterator with every file at `path`.
  ///
  /// Subfolders are indicated by a trailing slash '/' in filename. list_files() does not recurse
  /// into subfolders.
  pub fn list_files(&self, path: &str) -> Result<ListFilesIterator, Error> {
    ListFilesIterator::new(self.state, path)
      .or_else(|e| Err(format!("{} (Playdate: {})", e, self.last_err()).into()))
  }
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
