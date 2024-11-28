use core::fmt;
use shim::const_assert_size;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct TrapFrame {
    pub pc : u64,
    pub pstate : u64,
    pub sp : u64,
    pub tpidr : u64,
    pub ttbr0_el1 : u64,
    pub ttbr1_el1 : u64,
    big_regs : [u128; 32],
    pub regs : [u64; 31],
    reserved : u64,
}

const_assert_size!(TrapFrame, 816);
