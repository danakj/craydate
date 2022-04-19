use core::alloc::Layout;
use core::ffi::c_void;
use core::ptr::null_mut;

use static_assertions::*;

/// Compute how much space needs to be allocated such that the data can be aligned in that space.
///
/// This size has to fit the data after we align it, no matter what address the Playdate
/// allocator returns. As well, we have to fit a usize in front of the data, while keeping the
/// data aligned.
const fn calc_alloc_size(size: usize, align: usize) -> usize {
  let shift_storage_size = core::mem::size_of::<usize>();
  // Alignment of the data can require shifting up to `alignment - 1` many bytes. If
  // it would require `alignment` bytes, then it would actually not need to move. The shift is
  // computed % align.
  let alloc_size = size + (align - 1);
  // The most we need to move the data after alignment is `shift_storage_size`. So assume we have
  // to move it that much. Technically we could probably do something more complicated here to
  // save some bytes, because if the data was not shifted, we have up to `alignment - 1` extra
  // bytes allocated for the unused shift.
  if shift_storage_size % align == 0 {
    // `shift_storage_size` is a multiple of the alignment so just add it.
    (alloc_size + shift_storage_size) as usize
  } else {
    let aligned = ((shift_storage_size / align) + 1) * align;
    (alloc_size + aligned) as usize
  }
}

const fn calc_shift_for_align(addr: u64, align: usize) -> usize {
  let shift_storage_size = core::mem::size_of::<usize>() as u64;
  let align = align as u64;
  // We need to return a pointer aligned to `align`, but the alloc_fn() doesn't
  // promise any alignment. So we over-allocate `align` bytes in order to push the pointer
  // ahead as much as we need to. But then how do we know which pointer to give to free
  // later, if we moved it here? We *always* move the pointer ahead at least size_of::<usize>()
  // byte. If the returned pointer was aligned, we just shift it up by `align`. Then, in the
  // 8 bytes before the pointer, we store how many bytes we shifted the pointer in order to
  // recover that in dealloc().
  let shift = align - addr % align;
  if shift >= shift_storage_size {
    shift as usize
  } else {
    let needed = shift_storage_size - shift;
    if needed % align == 0 {
      (shift + needed) as usize
    } else {
      let aligned_needed = ((needed + align) / align) * align;
      (shift + aligned_needed) as usize
    }
  }
}

pub struct Allocator {
  sys: Option<&'static playdate_sys::playdate_sys>,
}

impl Allocator {
  pub const fn new() -> Allocator {
    Allocator::tests();
    Allocator { sys: None }
  }

  pub fn set_system_ptr(&mut self, sys: &'static playdate_sys::playdate_sys) {
    self.sys = Some(sys)
  }

  fn alloc_fn(&self, ptr: *mut u8, size: usize) -> *mut u8 {
    let sys = self.sys.unwrap();
    let realloc = sys.realloc.unwrap();
    unsafe { realloc(ptr as *mut c_void, size as u64) as *mut u8 }
  }

  fn write_shift_behind_ptr(ptr: *mut u8, shift: usize) {
    unsafe {
      core::ptr::write_unaligned(ptr.sub(core::mem::size_of::<usize>()) as *mut usize, shift)
    }
  }

  fn read_shift_behind_ptr(ptr: *mut u8) -> usize {
    unsafe { core::ptr::read_unaligned(ptr.sub(core::mem::size_of::<usize>()) as *mut usize) }
  }

  const fn tests() {
    const _STORAGE: usize = core::mem::size_of::<usize>();
    const_assert!(_STORAGE == 4 || _STORAGE == 8);

    // Alignment of 1 means nothing has to shift.
    const_assert_eq!(calc_alloc_size(1, 1), _STORAGE + 1);
    // Alignment is smaller than storage size and alloc size, so neither is aligned.
    const_assert!(_STORAGE != 4 || (calc_alloc_size(3, 2) == (2 * 2) + 3 + (2 - 1)));
    const_assert!(_STORAGE != 8 || (calc_alloc_size(3, 2) == (2 * 4) + 3 + (2 - 1)));
    // Alignment is larger than storage size and alloc size, but neither is aligned.
    const_assert_eq!(calc_alloc_size(5, 11), (11 * 1) + 5 + (11 - 1));
    // Storage size is aligned, alloc size is not.
    const_assert_eq!(calc_alloc_size(1, 4), _STORAGE + 1 + (4 - 1));
    const_assert_eq!(calc_alloc_size(2, 4), _STORAGE + 2 + (4 - 1));
    const_assert_eq!(calc_alloc_size(5, 4), _STORAGE + 5 + (4 - 1));
    // Storage size is not aligned, and is smaller than alignment. Alloc size is aligned.
    const_assert!(_STORAGE != 4 || (calc_alloc_size(5, 5) == (5 * 1) + 5 + (5 - 1)));
    const_assert!(_STORAGE != 8 || (calc_alloc_size(5, 5) == (5 * 2) + 5 + (5 - 1)));
    const_assert_eq!(calc_alloc_size(5, 20), (20 * 1) + 5 + (20 - 1));
    // Storage size is not aligned, and is larger than alignment. Alloc size is aligned.
    const_assert!(_STORAGE != 4 || (calc_alloc_size(5, 3) == (3 * 2) + 5 + (3 - 1)));
    const_assert!(_STORAGE != 8 || (calc_alloc_size(5, 3) == (3 * 3) + 5 + (3 - 1)));

    // Verify that the shifted data will fit in the allocated size for various sizes,
    // alignments, and allocation offsets.
    const_assert!(calc_shift_for_align(0, 1) <= calc_alloc_size(1000, 1) - 1000);
    const_assert!(calc_shift_for_align(1, 1) <= calc_alloc_size(1000, 1) - 1000);
    const_assert!(calc_shift_for_align(2, 1) <= calc_alloc_size(1000, 1) - 1000);
    const_assert!(calc_shift_for_align(3, 1) <= calc_alloc_size(1000, 1) - 1000);
    const_assert!(calc_shift_for_align(0, 4) <= calc_alloc_size(1000, 4) - 1000);
    const_assert!(calc_shift_for_align(1, 4) <= calc_alloc_size(1000, 4) - 1000);
    const_assert!(calc_shift_for_align(2, 4) <= calc_alloc_size(1000, 4) - 1000);
    const_assert!(calc_shift_for_align(3, 4) <= calc_alloc_size(1000, 4) - 1000);
    const_assert!(calc_shift_for_align(4, 4) <= calc_alloc_size(1000, 4) - 1000);
    const_assert!(calc_shift_for_align(5, 4) <= calc_alloc_size(1000, 4) - 1000);
    const_assert!(calc_shift_for_align(0, 1000) <= calc_alloc_size(1000, 1000) - 1000);
    const_assert!(calc_shift_for_align(1, 1000) <= calc_alloc_size(1000, 1000) - 1000);
    const_assert!(calc_shift_for_align(2, 1000) <= calc_alloc_size(1000, 1000) - 1000);
    const_assert!(calc_shift_for_align(3, 1000) <= calc_alloc_size(1000, 1000) - 1000);
    const_assert!(calc_shift_for_align(999, 1000) <= calc_alloc_size(1000, 1000) - 1000);
    const_assert!(calc_shift_for_align(1000, 1000) <= calc_alloc_size(1000, 1000) - 1000);
    const_assert!(calc_shift_for_align(1001, 1000) <= calc_alloc_size(1000, 1000) - 1000);
    // Alloc size < storage size.
    const_assert!(calc_shift_for_align(0, 8) <= calc_alloc_size(3, 8) - 3);
    const_assert!(calc_shift_for_align(1, 8) <= calc_alloc_size(3, 8) - 3);
    const_assert!(calc_shift_for_align(7, 8) <= calc_alloc_size(3, 8) - 3);
    const_assert!(calc_shift_for_align(8, 8) <= calc_alloc_size(3, 8) - 3);
    const_assert!(calc_shift_for_align(9, 8) <= calc_alloc_size(3, 8) - 3);
    // Alignment < storage size.
    const_assert!(calc_shift_for_align(0, 3) <= calc_alloc_size(100, 3) - 100);
    const_assert!(calc_shift_for_align(1, 3) <= calc_alloc_size(100, 3) - 100);
    const_assert!(calc_shift_for_align(2, 3) <= calc_alloc_size(100, 3) - 100);
    const_assert!(calc_shift_for_align(3, 3) <= calc_alloc_size(100, 3) - 100);
    const_assert!(calc_shift_for_align(4, 3) <= calc_alloc_size(100, 3) - 100);
    const_assert!(calc_shift_for_align(0, 3) <= calc_alloc_size(9, 3) - 9);
    const_assert!(calc_shift_for_align(1, 3) <= calc_alloc_size(9, 3) - 9);
    const_assert!(calc_shift_for_align(2, 3) <= calc_alloc_size(9, 3) - 9);
    const_assert!(calc_shift_for_align(3, 3) <= calc_alloc_size(9, 3) - 9);
    const_assert!(calc_shift_for_align(4, 3) <= calc_alloc_size(9, 3) - 9);
    const_assert!(calc_shift_for_align(8, 3) <= calc_alloc_size(9, 3) - 9);
    const_assert!(calc_shift_for_align(9, 3) <= calc_alloc_size(9, 3) - 9);
    const_assert!(calc_shift_for_align(10, 3) <= calc_alloc_size(9, 3) - 9);
  }
}

#[cfg(not(doc))]
unsafe impl core::alloc::GlobalAlloc for Allocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let size = calc_alloc_size(layout.size(), layout.align());
    let ptr = self.alloc_fn(null_mut(), size) as *mut u8;
    let shift = calc_shift_for_align(ptr as u64, layout.align());

    assert!(layout.size() + shift <= size);
    assert_eq!(ptr.add(shift) as usize % layout.align(), 0);

    let ptr = ptr.add(shift);
    Self::write_shift_behind_ptr(ptr, shift);
    ptr
  }

  unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
    let shift = core::ptr::read_unaligned(ptr.sub(core::mem::size_of::<usize>()) as *mut usize);
    self.alloc_fn(ptr.sub(shift), 0);
  }

  unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    let old_shift = Self::read_shift_behind_ptr(ptr);

    let size = calc_alloc_size(new_size, layout.align());
    let ptr = self.alloc_fn(ptr.sub(old_shift), size);
    let new_shift = calc_shift_for_align(ptr as u64, layout.align());

    assert!(layout.size() + new_shift < size);
    assert_eq!(ptr.add(new_shift) as usize % layout.align(), 0);

    let ptr = ptr.add(new_shift);
    Self::write_shift_behind_ptr(ptr, new_shift);
    ptr
  }
}
