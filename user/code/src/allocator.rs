extern crate heap;
use heap::align_up;

use alloc::alloc::Layout;
use alloc::alloc::GlobalAlloc;
use heap::{AllocatorImpl, LocalAlloc};
use core::fmt;
use spin::Mutex;

use crate::uprintln;
/// Thread-safe (locking) wrapper around a particular memory allocator.
pub struct Allocator(Mutex<Option<AllocatorImpl>>);

impl Allocator {
    /// Returns an uninitialized `Allocator`.
    ///
    /// The allocator must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        Allocator(Mutex::new(None))
    }

    /// Initializes the memory allocator.
    /// The caller should assure that the method is invoked only once during the
    /// kernel initialization.
    ///
    /// # Panics
    ///
    /// Panics if the system's memory map could not be retrieved.
    pub unsafe fn initialize(&self, start: usize, end: usize) {
        for i in start..=end {
            let ptr = i as *mut u8;
            unsafe {
                *ptr = 0;
            }
        }
        *self.0.lock() = Some(AllocatorImpl::new(start, end));
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let aligned_layout = Layout::from_size_align(layout.size(), layout.align().max(4))
            .expect("Invalid layout for allocation");
        let ptr = self.0
            .lock()
            .as_mut()
            .expect("allocator uninitialized")
            .alloc(aligned_layout);
        //uprintln!("alloc {:x}, {:#?}", ptr as u64, layout);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        //uprintln!("dealloc {:x}, {:#?}", ptr as u64, layout);
        let aligned_layout = Layout::from_size_align(layout.size(), layout.align().max(4))
            .expect("Invalid layout for deallocation");
        self.0
            .lock()
            .as_mut()
            .expect("allocator uninitialized")
            .dealloc(ptr, aligned_layout);
    }
}


impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.lock().as_mut() {
            Some(ref alloc) => {
                write!(f, "{:?}", alloc)?;
            }
            None => write!(f, "Not yet initialized")?,
        }
        Ok(())
    }
}
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

extern "C" {
    static __text_end: u8;
}
const PAGE_SIZE : usize = 64 * 1024;
const USER_IMG_BASE : usize = 0xffff_ffff_c000_0000;
const USER_HEAP_PAGES : usize = 2;


/// Returns the (start address, end address) of the available memory on this
/// system if it can be determined. If it cannot, `None` is returned.
///
/// This function is expected to return `Some` under all normal cirumstances.
pub fn memory_map() -> (usize, usize) {
    let mut binary_end = unsafe { (&__text_end as *const u8) as usize };
    binary_end = align_up(binary_end, PAGE_SIZE);
    (binary_end, binary_end + USER_HEAP_PAGES*PAGE_SIZE - 1)
}

