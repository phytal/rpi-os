#![cfg_attr(not(test), no_std)]
#![feature(decl_macro)]
#![feature(strict_overflow_ops)]
mod linked_list;
mod util;
pub use self::util::{align_down, align_up};

mod bin;
mod bump;

pub type AllocatorImpl = bin::Allocator;

#[cfg(test)]
mod tests;

use core::alloc::Layout;

/// `LocalAlloc` is an analogous trait to the standard library's `GlobalAlloc`,
/// but it takes `&mut self` in `alloc()` and `dealloc()`.
pub trait LocalAlloc {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
}

