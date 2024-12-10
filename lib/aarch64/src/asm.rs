/// Wait for event not to burn CPU.

use core::arch::asm;
#[inline(always)]
pub fn wfe() {
    unsafe { asm!("wfe") }; //  volatile is the default 
}

/// Wait for interrupt not to burn CPU.
#[inline(always)]
pub fn wfi() {
    unsafe { asm!("wfi") };
}

/// A NOOP that won't be optimized out.
#[inline(always)]
pub fn nop() {
    unsafe { asm!("nop") };
}

/// Transition to a lower level
#[inline(always)]
pub fn eret() {
    unsafe { asm!("eret") };
}

/// Instruction Synchronization Barrier
#[inline(always)]
pub fn isb() {
    unsafe { asm!("isb") };
}

/// Data Synchronization Barrier
#[inline(always)]
pub fn dsb() {
    unsafe { asm!("dsb SY") };
}

/// Set Event
#[inline(always)]
pub fn sev() {
    unsafe { asm!("sev") };
}

/// Enable (unmask) interrupts
#[inline(always)]
pub unsafe fn sti() {
    asm!("msr DAIFClr, 0b0010");
}

/// Disable (mask) interrupt
#[inline(always)]
pub unsafe fn cli() {
    asm!("msr DAIFSet, 0b0010");
}

/// Break with an immeidate
#[macro_export]
macro_rules! brk {
    ($num:tt) => {
        unsafe { asm!(concat!("brk ", stringify!($num))); }
    }
}

/// Supervisor call with an immediate
#[macro_export]
macro_rules! svc {
    ($num:tt) => {
        unsafe { asm!(concat!("svc ", stringify!($num))); }
    }
}
