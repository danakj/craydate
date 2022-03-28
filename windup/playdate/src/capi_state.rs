use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use core::ptr::NonNull;

use crate::ctypes::*;
use crate::executor::Executor;
use crate::bitmap::Bitmap;

#[non_exhaustive]
#[derive(Debug)]
pub struct CApiState {
  pub capi: &'static CApi,
  pub cdisplay: &'static CDisplay,
  pub csystem: &'static CSystem,
  pub cfile: &'static CFile,
  pub cgraphics: &'static CGraphics,
  pub csound: &'static CSound,
  pub executor: NonNull<Executor>,

  pub frame_number: Cell<u64>,
  pub peripherals_enabled: Cell<PDPeripherals>,
  // Tracks the button state for the current and previous frame respectively.
  pub button_state_per_frame: Cell<[Option<PDButtonsSet>; 2]>,
  pub stack: RefCell<ContextStack>,
  // Tracks how many times the stencil was set.
  pub stencil_generation: Cell<usize>,
  // Tracks how many times the font was set.
  pub font_generation: Cell<usize>,
}
impl CApiState {
  pub fn new(capi: &'static CApi) -> CApiState {
    CApiState {
      cgraphics: unsafe { &*capi.graphics },
      csystem: unsafe { &*capi.system },
      cdisplay: unsafe { &*capi.display },
      cfile: unsafe { &*capi.file },
      csound: unsafe { &* capi.sound },
      capi,
      executor: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Executor::new()))) },
      frame_number: Cell::new(0),
      peripherals_enabled: Cell::new(PDPeripherals::kNone),
      button_state_per_frame: Cell::new([None, None]),
      stack: RefCell::new(ContextStack::new()),
      stencil_generation: Cell::new(0),
      font_generation: Cell::new(0),
    }
  }

  /// Stores the current frame's button states, and moves the previous frames' states into the
  /// next position.
  pub fn set_current_frame_button_state(&self, buttons_set: PDButtonsSet) {
    let mut buttons = self.button_state_per_frame.take();
    // On the first frame, we push a duplicate frame.
    if let None = buttons[0] {
      buttons[0] = Some(buttons_set);
    }
    // Move the "current" slot to the "last frame" slot.
    buttons[1] = buttons[0];
    // Save the current frame.
    buttons[0] = Some(buttons_set);
    self.button_state_per_frame.set(buttons);
  }

  pub fn reset_context_stack(&self) {
    *self.stack.borrow_mut() = ContextStack::new();
  }
}

/// Holds a reference on an Bitmap that was placed into the context stack. The reference can
/// be used to retrieve that Bitmap on a future frame, after it is released.
#[derive(Debug)]
pub struct ContextStackId {
  state: &'static CApiState,
  id: usize,
}
impl Clone for ContextStackId {
  fn clone(&self) -> Self {
    let mut stack = self.state.stack.borrow_mut();
    let held = stack.holding.get_mut(&self.id);
    let held = unsafe { held.unwrap_unchecked() };
    held.refs += 1;

    ContextStackId {
      state: self.state,
      id: self.id,
    }
  }
}
impl Drop for ContextStackId {
  fn drop(&mut self) {
    let mut stack = self.state.stack.borrow_mut();
    let held = stack.holding.get_mut(&self.id);
    let held = unsafe { held.unwrap_unchecked() };
    held.refs -= 1;
    if held.refs == 0 {
      stack.holding.remove(&self.id);
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

#[derive(Debug)]
struct HeldBitmap {
  refs: usize,
  bitmap: Option<Bitmap>,
}

#[derive(Debug)]
struct StackBitmap {
  id: usize,
  bitmap: Bitmap,
}

#[derive(Debug)]
pub struct ContextStack {
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
  fn new() -> Self {
    ContextStack {
      stack: Vec::new(),
      holding: BTreeMap::new(),
    }
  }

  pub fn push_framebuffer(&mut self, state: &'static CApiState) {
    unsafe { state.cgraphics.pushContext.unwrap()(core::ptr::null_mut()) };

    self.stack.push(None)
  }
  pub fn push_bitmap(&mut self, state: &'static CApiState, mut bitmap: Bitmap) -> ContextStackId {
    unsafe { state.cgraphics.pushContext.unwrap()(bitmap.as_bitmap_mut_ptr()) };

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
    ContextStackId {
      state: state,
      id,
    }
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
          Some(ContextStackId {
            state,
            id: stack_b.id,
          })
        }
      }
    })
  }
  pub fn take_bitmap(&mut self, id: ContextStackId) -> Option<Bitmap> {
    self.holding.remove(&id.id).and_then(|held| held.bitmap)
  }

}
