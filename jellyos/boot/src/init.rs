use aarch64::*;
use core::mem::zeroed;
use core::ptr::write_volatile;
mod panic;

use crate::bootloader;
use core::arch::global_asm;

global_asm!(include_str!("init/init.s"));

#[allow(static_mut_refs)]
unsafe fn zeros_bss() {
    extern "C" {
        static mut __bss_beg: u64;
        static mut __bss_end: u64;
    }

    let mut iter: *mut u64 = &mut __bss_beg;
    let end: *mut u64 = &mut __bss_end;

    while iter < end {
        write_volatile(iter, zeroed());
        iter = iter.add(1);
    }
}
/// Kernel entrypoint for core 0
#[no_mangle]
pub unsafe extern "C" fn kinit() -> ! {
    if MPIDR_EL1.get_value(MPIDR_EL1::Aff0) == 0 {
        SP.set(crate::BINARY_START_ADDR);
        zeros_bss();
        bootloader();
    }
    unreachable!();
}