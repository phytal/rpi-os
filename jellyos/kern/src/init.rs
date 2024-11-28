use aarch64::*;

use core::arch::global_asm;
use core::mem::zeroed;
use core::ptr::addr_of;
use core::ptr::write_volatile;

mod oom;
mod panic;

use crate::console::kprintln;
use crate::kmain;
use crate::param::*;
use crate::per_core_main;
use pi::timer::spin_sleep;
use core::time::Duration;

global_asm!(include_str!("init/vectors.s"));

//
// big assumptions (better to be checked):
//   _start1/2(), _kinit1/2(), switch_to_el1/2() should NOT use stack!
//   e.g., #[no_stack] would be useful ..
//
// so, no debug build support!
//

/// Kernel entrypoint for core 0
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    if MPIDR_EL1.get_value(MPIDR_EL1::Aff0) == 0 {
        SP.set(KERN_STACK_BASE);
        kinit();
    }
    loop {}
}

// symbol table has vars pointing to bss
// take bss_beg, _end labels from linker (.ld) and manually set to 0
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

#[no_mangle]
unsafe fn switch_to_el2() {
    if current_el() == 3 {

        SP_EL2.set(SP.get() as u64);

        // set up Secure Configuration Register (D13.2.10)
        SCR_EL3.set(SCR_EL3::NS | SCR_EL3::SMD | SCR_EL3::HCE | SCR_EL3::RW | SCR_EL3::RES1);

        // set up Saved Program Status Register (C5.2.19)
        SPSR_EL3
            .set((SPSR_EL3::M & 0b1001) | SPSR_EL3::F | SPSR_EL3::I | SPSR_EL3::A | SPSR_EL3::D);

        // eret to itself, expecting current_el() == 2 this time.
        ELR_EL3.set(switch_to_el2 as u64);
        asm::eret();
    }
}


#[no_mangle]
unsafe fn switch_to_el1() {
    extern "C" {
        static mut vectors: u64;
    }

    if current_el() == 2 {
        // set the stack-pointer for EL1
        SP_EL1.set(SP.get() as u64);

        // enable CNTP for EL1/EL0 (ref: D7.5.2, D7.5.13)
        // NOTE: This doesn't actually enable the counter stream.
        CNTHCTL_EL2.set(CNTHCTL_EL2.get() | CNTHCTL_EL2::EL0VCTEN | CNTHCTL_EL2::EL0PCTEN);
        CNTVOFF_EL2.set(0);

        // enable AArch64 in EL1 (A53: 4.3.36)
        HCR_EL2.set(HCR_EL2::RW | HCR_EL2::RES1);

        // enable floating point and SVE (SIMD) (A53: 4.3.38, 4.3.34)
        CPTR_EL2.set(0);
        CPACR_EL1.set(CPACR_EL1.get() | (0b11 << 20));

        // Set SCTLR to known state (A53: 4.3.30)
        SCTLR_EL1.set(SCTLR_EL1::RES1);

        // set up exception handlers
        VBAR_EL1.set(addr_of!(vectors) as u64);

        // change execution level to EL1 (ref: C5.2.19)
        SPSR_EL2.set(
            (SPSR_EL2::M & 0b0101) // EL1h // selects stack pointer and return to exception level 1
            | SPSR_EL2::F
            | SPSR_EL2::I
            | SPSR_EL2::D
            | SPSR_EL2::A,
        );

        ELR_EL2.set(switch_to_el1 as u64);
        asm::eret();
    }
}

#[no_mangle]
unsafe fn kinit() -> ! {
    spin_sleep(Duration::from_millis(500)); // necessary delay after transmit before tty
    kprintln!("What just happened? Why am I here?");
    kprintln!("Discovering my identity and features!");
    kprintln!("Initial Exception Level: {:x}", current_el());    
    zeros_bss();

    switch_to_el2();
    switch_to_el1();
    kmain();
}

/// Kernel entrypoint for core 1, 2, and 3
#[no_mangle]
pub unsafe extern "C" fn start2() -> ! {
    // Lab 5 1.A
    SP.set(KERN_STACK_BASE - KERN_STACK_SIZE * MPIDR_EL1.get_value(MPIDR_EL1::Aff0) as usize);
    kinit2();
}

unsafe fn kinit2() -> ! {
    switch_to_el2();
    switch_to_el1();
    kmain2();
}

unsafe fn kmain2() -> ! {
    // Lab 5 1.A
    let core_num = MPIDR_EL1.get_value(MPIDR_EL1::Aff0) as usize;
    SPINNING_BASE.add(core_num).write_volatile(0);

    per_core_main();
}

/// Wakes up each app core by writing the address of `init::start2`
/// to their spinning base and send event with `sev()`.
pub unsafe fn initialize_app_cores() {
    // Lab 5 1.A
    for core_num in 1..NCORES {
        SPINNING_BASE.add(core_num).write_volatile(self::start2 as usize);
    }
    sev();
    for core_num in 1..NCORES {
        while SPINNING_BASE.add(core_num).read_volatile() != 0 {}
    }
}
