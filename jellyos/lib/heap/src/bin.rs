use core::alloc::Layout;

use crate::linked_list::LinkedList;
use crate::util::*;
use crate::LocalAlloc;

/// A simple allocator that allocates based on size classes.
///   bin 0    : handles allocations in (0, 2^3]
///   bin 1    : handles allocations in (2^3, 2^4]
///   ...
///   bin 29    : handles allocations in (2^31, 2^32]
///   
///   map_to_bin(size) -> k
///   

#[derive(Debug)]
pub struct Allocator {
    current: usize,
    end: usize,
    bins: [LinkedList; 32 - 2],
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        Allocator {
            current: start,
            end,
            bins: [LinkedList::new(); 32 - 2],
        }
    }
}
use core::cmp::max;

// used the bins of increasing 2 powers
impl LocalAlloc for Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning null pointer (`core::ptr::null_mut`)
    /// indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's
    /// size or alignment constraints.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let idx = if layout.size() == 0 {
            0
        } else {
            let size_minus_one = layout.size().saturating_sub(1); // Use saturating_sub to prevent underflow
            let ilog = if size_minus_one > 0 { size_minus_one.ilog2() as i32 } else { 0 };
            max(0, ilog - 2) as usize // Ensure the result is non-negative
        };
        match self.bins[idx]
            .iter_mut()
            .find(|x| ((x.value() as usize) % layout.align()) == 0)
        {
            Some(node) => {
                node.pop() as *mut u8
            },
            _ => {
                let potential_addr = align_up(self.current, layout.align());
                match potential_addr.checked_add(1 << (idx + 3)) {
                    Some(new_current) if new_current <= self.end => {
                        self.current = new_current;
                        potential_addr as *mut u8
                    }
                    _ => core::ptr::null_mut(),
                }
            }
        }
    }
    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let idx = if layout.size() == 0 {
            0
        } else {
            let size_minus_one = layout.size().saturating_sub(1); // Use saturating_sub to prevent underflow
            let ilog = if size_minus_one > 0 { size_minus_one.ilog2() as i32 } else { 0 };
            max(0, ilog - 2) as usize // Ensure the result is non-negative
        };
        //                                                                     // FML
        //                                                                     // `LinkedList` guarantees that the passed in pointer refers to valid, unique,
        //                                                                     // writeable memory at least `usize` in size.
        //                                                                     // FMLpt
        //                                                                     sizeof(usize) == 8
        //                                                                     so log2(size) >=3
        self.bins[idx].push(ptr as *mut usize);
    }
}
