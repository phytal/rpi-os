use core::convert::TryInto;
use core::time::Duration;

use volatile::{ReadVolatile, Volatile};
use volatile::prelude::*;

use crate::common::IO_BASE;

/// The base address for the ARM system timer registers.
/// The Physical (hardware) base address for the system timers is 0x7E003000. 
const TIMER_REG_BASE: usize = IO_BASE + 0x3000;

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    CS: Volatile<u32>,
    CLO: ReadVolatile<u32>,
    CHI: ReadVolatile<u32>,
    COMPARE: [Volatile<u32>; 4],
}

/// The Raspberry Pi ARM system timer.
pub struct Timer {
    registers: &'static mut Registers,
}

impl Timer {
    /// Returns a new instance of `Timer`.
    pub fn new() -> Timer {
        Timer {
            registers: unsafe { &mut *(TIMER_REG_BASE as *mut Registers) },
        }
    }

    /// Reads the system timer's counter and returns Duration.
    /// `CLO` and `CHI` together can represent the number of elapsed microseconds.
    pub fn read(&self) -> Duration {
        let mut micros: u64 = self.registers.CLO.read() as u64;
        micros |= (self.registers.CHI.read() as u64) << 32;
        Duration::from_micros(micros)
    }
    
    /// Sets up a match in timer 1 to occur `t` duration from now. If
    /// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
    /// interrupt will be issued in `t` duration.
    pub fn tick_in(&mut self, t: Duration) {
        let time = self.read()+t;
        self.registers.CS.or_mask(0b10); // clear old detection status
        self.registers.COMPARE[1].write(time.as_micros() as u32); 
    }
}

/// Returns current time.
pub fn current_time() -> Duration {
    Timer::new().read()
}

/// Spins until `t` duration have passed.
pub fn spin_sleep(t: Duration) {
    let start = current_time();
    while start + t > current_time() {}
}

/// Sets up a match in timer 1 to occur `t` duration from now. If
/// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
/// interrupt will be issued in `t` duration.
pub fn tick_in(t: Duration) {
    Timer::new().tick_in(t);
}
