pub struct _SP;
use core::arch::asm;

impl _SP {
    /// Returns the current stack pointer.
    #[inline(always)]
    pub fn get(&self) -> usize {
        let rtn: usize;
        unsafe {
            asm!(
                "mov {0}, sp",
                out(reg) rtn,
                options(nomem, nostack, preserves_flags)
            );
        }
        rtn
    }

    /// Set the current stack pointer with a passed argument.
    #[inline(always)]
    pub unsafe fn set(&self, stack: usize) {
        asm!(
            "mov sp, {0}",
            in(reg) stack,
            options(nomem, nostack, preserves_flags)
        );
    }
}

pub static SP: _SP = _SP {};
