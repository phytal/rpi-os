#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
// features
#![allow(internal_features)]
#![feature(ptr_internals)]
#![feature(raw_vec_internals)]
#![feature(prelude_2024)]
#![feature(alloc_error_handler)]
#![feature(decl_macro)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(const_mut_refs)]
#![feature(const_option)]
#![feature(let_chains)]
#![feature(asm_const)]
#![feature(iter_chain)]
#![feature(if_let_guard)] // experimental
#![feature(array_chunks)] // experimental
// #[cfg(not(test))] // commenting for rust-analyzer
mod init;

extern crate alloc;
#[macro_use]
extern crate log;

pub mod allocator;
pub mod console;
pub mod fs;
pub mod logger;
pub mod mutex;
pub mod net;
pub mod param;
pub mod percore;
pub mod process;
pub mod shell;
pub mod traps;
pub mod vm;


extern crate heap;
use allocator::memory_map;
use allocator::Allocator;
use fs::FileSystem;
use net::uspi::Usb;
use net::GlobalEthernetDriver;
use process::GlobalScheduler;
use shell::shell;
use traps::irq::{Fiq, GlobalIrq, LocalIrq};
use vm::VMManager;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();
pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();
pub static VMM: VMManager = VMManager::uninitialized();
pub static USB: Usb = Usb::uninitialized();
pub static GLOBAL_IRQ: GlobalIrq = GlobalIrq::new();
pub static IRQ: LocalIrq = LocalIrq::new();
pub static FIQ: Fiq = Fiq::new();
pub static ETHERNET: GlobalEthernetDriver = GlobalEthernetDriver::uninitialized();

use crate::console::kprintln;
use pi::timer::spin_sleep;
use core::time::Duration;

extern "C" {
    static __text_beg: u64;
    static __text_end: u64;
    static __bss_beg: u64;
    static __bss_end: u64;
}

unsafe fn log_layout() {
    crate::logger::init_logger();

    info!(
        "text beg: {:016x}, end: {:016x}",
        &__text_beg as *const _ as u64, &__text_end as *const _ as u64
    );
    info!(
        "bss  beg: {:016x}, end: {:016x}",
        &__bss_beg as *const _ as u64, &__bss_end as *const _ as u64
    );
}

/// bootstrapping core
unsafe fn kmain() -> ! {

    spin_sleep(Duration::from_millis(500)); // necessary delay after transmit before tty
    log_layout();
    ALLOCATOR.initialize();
    FILESYSTEM.initialize();
    VMM.initialize();
    SCHEDULER.initialize();
    init::initialize_app_cores();

    per_core_main();
}

unsafe fn per_core_main() -> ! {
    VMM.wait();
    SCHEDULER.start();
}
