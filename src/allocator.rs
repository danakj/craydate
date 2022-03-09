use core::alloc::Layout;
use core::ffi::c_void;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, Ordering};

pub struct Allocator {
    sys: AtomicPtr<playdate_sys::playdate_sys>,
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            sys: AtomicPtr::new(null_mut()),
        }
    }

    pub fn set_system_ptr(&self, sys: *mut playdate_sys::playdate_sys) {
        self.sys.store(sys, Ordering::Release);
    }

    fn alloc_fn(&self, realloc: *mut u8, size: usize) -> *mut u8 {
        let sys = self.sys.load(Ordering::Acquire);
        let f = unsafe { (*sys).realloc }.unwrap();
        unsafe { f(realloc as *mut c_void, size as u64) as *mut u8 }
    }

    fn calc_alloc_size(size: usize, align: usize) -> usize {
        let shift_storage_size = core::mem::size_of::<usize>();
        size + shift_storage_size + align - 1
    }

    fn calc_shift_for_align(addr: u64, align: usize) -> usize {
        let shift_storage_size = core::mem::size_of::<usize>() as u64;
        let align = align as u64;
        // We need to return a pointer aligned to `align`, but the alloc_fn() doesn't
        // promise any alignment. So we over-allocate `align` bytes in order to push the pointer
        // ahead as much as we need to. But then how do we know which pointer to give to free
        // later, if we moved it here? We *always* move the pointer ahead at least size_of::<usize>()
        // byte. If the returned pointer was aligned, we just shift it up by `align`. Then, in the
        // 8 bytes before the pointer, we store how many bytes we shifted the pointer in order to
        // recover that in dealloc().
        ((align - ((addr + shift_storage_size) % align)) % align) as usize
    }

    fn write_shift_behind_ptr(ptr: *mut u8, shift: usize) {
        unsafe {
            core::ptr::write_unaligned(ptr.sub(core::mem::size_of::<usize>()) as *mut usize, shift)
        }
    }

    fn read_shift_behind_ptr(ptr: *mut u8) -> usize {
        unsafe { core::ptr::read_unaligned(ptr.sub(core::mem::size_of::<usize>()) as *mut usize) }
    }
}

unsafe impl core::alloc::GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = Self::calc_alloc_size(layout.size(), layout.align());
        let ptr = self.alloc_fn(null_mut(), size) as *mut u8;
        let shift = Self::calc_shift_for_align(ptr as u64, layout.align());
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

        let size = Self::calc_alloc_size(new_size, layout.align());
        let ptr = self.alloc_fn(ptr.sub(old_shift), size);
        let new_shift = Self::calc_shift_for_align(ptr as u64, layout.align());
        let ptr = ptr.add(new_shift);
        Self::write_shift_behind_ptr(ptr, new_shift);
        ptr
    }
}

/*
#if TARGET_PLAYDATE

#include "pd_api.h"

typedef int (PDEventHandler)(PlaydateAPI* playdate, PDSystemEvent event, uint32_t arg);

extern PDEventHandler eventHandler;
PDEventHandler* PD_eventHandler __attribute__((section(".capi_handler"))) = &eventHandler;

extern uint32_t bssStart asm("__bss_start__");
uint32_t* _bss_start __attribute__((section(".bss_start"))) = &bssStart;

extern uint32_t bssEnd asm("__bss_end__");
uint32_t* _bss_end __attribute__((section(".bss_end"))) = &bssEnd;

#endif
*/
