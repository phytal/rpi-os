use core::ops::BitAnd;

use aarch64::ESR_EL1;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Fault {
    AddressSize,
    Translation,
    AccessFlag,
    Permission,
    Alignment,
    TlbConflict,
    Other(u8),
}
#[allow(non_contiguous_range_endpoints)]
impl From<u32> for Fault {
    fn from(val: u32) -> Fault {
        use self::Fault::*;
        match val as u8 {
            0b000000..=0b000011 => AddressSize,
            0b000100..=0b000111 => Translation,
            0b001001..=0b001011 => AccessFlag,
            0b001101..=0b001111 => Permission,
            0b100001 => Alignment,
            0b110000 => TlbConflict,
            _ => Other(val as u8)
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    WfiWfe,
    SimdFp,
    IllegalExecutionState,
    Svc(u16),
    Hvc(u16),
    Smc(u16),
    MsrMrsSystem,
    InstructionAbort { kind: Fault, level: u8 },
    PCAlignmentFault,
    DataAbort { kind: Fault, level: u8 },
    SpAlignmentFault,
    TrappedFpu,
    SError,
    Breakpoint,
    Step,
    Watchpoint,
    Brk(u16),
    Other(u32),
}

/// Converts a raw syndrome value (ESR) into a `Syndrome` (ref: D1.10.4).
impl From<u32> for Syndrome {
    fn from(esr: u32) -> Syndrome {
        use self::Syndrome::*;
        match (esr >> 26).bitand(0x3F) {
            0b000000 => Unknown,
            0b000001 => WfiWfe,
            0b000111 => SimdFp,
            0b001110 => IllegalExecutionState,
            0b010101 | 0b010001 => Svc(ESR_EL1::get_value(esr as u64, ESR_EL1::ISS_HSVC_IMM) as u16),
            0b010110 | 0b010010 => Hvc(ESR_EL1::get_value(esr as u64, ESR_EL1::ISS_HSVC_IMM) as u16),
            0b10111 | 0b010011 => Smc(ESR_EL1::get_value(esr as u64, ESR_EL1::ISS_HSVC_IMM) as u16),
            0b011000 => MsrMrsSystem,
            0b100000 | 0b100001 => InstructionAbort { kind: Fault::from(esr & 0x1F), level: (esr & 0b11) as u8 },
            0b100010 => PCAlignmentFault,
            0b100100 | 100101 => DataAbort { kind: Fault::from(esr & 0x1F), level: (esr & 0b11) as u8 },
            0b100110 => SpAlignmentFault,
            0b101000 | 0b101100 => TrappedFpu,
            0b101111 => SError,
            0b110000 | 0b110001 => Breakpoint,
            0b110010 | 0b110011 => Step,
            0b110100 | 0b110101 => Watchpoint,
            0b111100 => Brk(ESR_EL1::get_value(esr as u64, ESR_EL1::ISS_BRK_CMMT) as u16),
            _ => Other(esr),
        }
    }
}
