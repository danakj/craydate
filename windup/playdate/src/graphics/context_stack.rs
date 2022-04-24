use alloc::{collections::BTreeMap, vec::Vec};

use super::bitmap::Bitmap;
use crate::capi_state::CApiState;

#[derive(Debug)]
struct StackBitmap {
  id: usize,
  bitmap: Bitmap,
}

#[derive(Debug)]
struct HeldBitmap {
  refs: usize,
  bitmap: Option<Bitmap>,
}

#[derive(Debug)]
pub(crate) struct ContextStack {
  /// The active stack. The top of the stack is where drawing commands are currently being applied.
  /// A None at the top of the stack refers to the framebuffer, and an empty stack also refers to
  /// the framebuffer.
  stack: Vec<Option<StackBitmap>>,

  /// Bitmaps that were dropped from the stack (at the end of a frame), which are kept around to be
  /// reclaimed. The key is an id referring to the bitmap, which the game can hold. The bitmap is
  /// None until that bitmap is dropped, then it holds the bitmap. If the id is removed from the
  /// map before the bitmap, it will never be held in this map.
  holding: BTreeMap<usize, HeldBitmap>,
}
impl ContextStack {
  pub fn new() -> Self {
    ContextStack {
      stack: Vec::new(),
      holding: BTreeMap::new(),
    }
  }

  pub fn push_framebuffer(&mut self) {
    unsafe { Self::fns().pushContext.unwrap()(core::ptr::null_mut()) };

    self.stack.push(None)
  }
  pub fn push_bitmap(&mut self, bitmap: Bitmap) -> ContextStackId {
    // pushContext() takes a mutable pointer but does not change the data inside it.
    unsafe { Self::fns().pushContext.unwrap()(bitmap.cptr() as *mut _) };

    static mut NEXT_ID: usize = 1;
    let id = unsafe {
      let id = NEXT_ID;
      NEXT_ID += 1;
      id
    };
    self.stack.push(Some(StackBitmap { id, bitmap }));
    self.holding.insert(
      id,
      HeldBitmap {
        refs: 1,
        bitmap: None,
      },
    );
    ContextStackId { id }
  }
  pub fn pop(&mut self, state: &'static CApiState) -> Option<ContextStackId> {
    unsafe { state.cgraphics.popContext.unwrap()() };

    // If the back of the stack is a StackBitmap, then unwrap that.
    self.stack.pop().and_then(|last| last).and_then(|stack_b| {
      // Verify if we're keeping a space around for the popped bitmap, otherwise we just drop
      // the bitmap.
      match self.holding.get_mut(&stack_b.id) {
        // The last ContextStackId was already dropped.
        None => None,
        // We have a spot for the bitmap to be held, so we insert it in there, and construct another
        // reference to it (as a ContextStackId).
        Some(held) => {
          assert!(held.bitmap.is_none());
          held.bitmap = Some(stack_b.bitmap);
          assert!(held.refs >= 1);
          held.refs += 1;
          Some(ContextStackId { id: stack_b.id })
        }
      }
    })
  }
  pub fn take_bitmap(&mut self, id: ContextStackId) -> Option<Bitmap> {
    let r = self.holding.remove(&id.id).and_then(|held| held.bitmap);
    // We can forget the ContextStackId as no id can refer to the bitmap once it's removed, and we
    // are no longer tracking counts of outstanding ContextStackIds at that point.
    core::mem::forget(id);
    r
  }

  pub fn fns() -> &'static playdate_sys::playdate_graphics {
    CApiState::get().cgraphics
  }
}

/// Holds a reference on an Bitmap that was placed into the context stack. The reference can
/// be used to retrieve that Bitmap on a future frame, after it is released.
#[derive(Debug)]
pub struct ContextStackId {
  id: usize,
}
impl Clone for ContextStackId {
  fn clone(&self) -> Self {
    let mut stack = CApiState::get().stack.borrow_mut();
    let held = stack.holding.get_mut(&self.id);
    let held = unsafe { held.unwrap_unchecked() };
    held.refs += 1;

    ContextStackId { id: self.id }
  }
}
impl Drop for ContextStackId {
  fn drop(&mut self) {
    let mut stack = CApiState::get().stack.borrow_mut();
    match stack.holding.get_mut(&self.id) {
      // The bitmap is still held in the stack, and may be moved with another ContextStackId if
      // there are any left.
      Some(held) => {
        held.refs -= 1;
        if held.refs == 0 {
          stack.holding.remove(&self.id);
        }
      }
      // In this case, take_bitmap() was called so the id is not in the map anymore.
      None => (),
    }
  }
}

impl PartialEq for ContextStackId {
  fn eq(&self, other: &Self) -> bool {
    self.id.eq(&other.id)
  }
}
impl Eq for ContextStackId {}
impl PartialOrd for ContextStackId {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    self.id.partial_cmp(&other.id)
  }
}
impl Ord for ContextStackId {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.id.cmp(&other.id)
  }
}
impl core::hash::Hash for ContextStackId {
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
  }
}
