use pi::atags::Atags;
use crate::mutex::Mutex;
use core::fmt;
use alloc::alloc::Layout;
use alloc::alloc::GlobalAlloc;

extern "C" {
    static __text_end: u8;
}

use heap::{AllocatorImpl, LocalAlloc, align_up};

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
    pub unsafe fn initialize(&self) {
        let (start, end) = memory_map().expect("failed to get memory map");
        info!("heap beg: {:016x}, end: {:016x}", start, end);
        *self.0.lock() = Some(AllocatorImpl::new(start, end));
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // kprintln!("allocing: {}  bytes", layout.size());
        let aligned_layout = Layout::from_size_align(layout.size(), layout.align().max(4))
            .expect("Invalid layout for allocation");
        self.0
            .lock()
            .as_mut()
            .expect("allocator uninitialized")
            .alloc(aligned_layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // kprintln!("deallocing: {}  bytes", layout.size());
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
/// Returns the (start address, end address) of the available memory on this
/// system if it can be determined. If it cannot, `None` is returned.
///
/// This function is expected to return `Some` under all normal cirumstances.
pub fn memory_map() -> Option<(usize, usize)> {
    
    let page_size: usize = 1 << 12;
    let mut binary_end = unsafe { (&__text_end as *const u8) as usize };
    binary_end = align_up(binary_end, page_size);
    let mut atags = Atags::get();
    match atags.find(|tag| tag.mem().is_some()) {
        Some(atag) => {
            let mem = atag.mem().unwrap();
            Some((binary_end, (mem.size as usize) - binary_end))
        }
        None => Some((1000000, 1006022656)) // atags not appearing for ELF kernel QEMU fix,
        // None => None // correct code
    }

}

