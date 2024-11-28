use crate::common::IO_BASE;

use shim::const_assert_size;
use volatile::prelude::*;
use volatile::{ReadVolatile, Volatile};

// "The base address for the ARM interrupt register is 0x7E00B000.""
const INT_BASE: usize = IO_BASE + 0xB000 + 0x200;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Interrupt {
    Timer1 = 1,
    Timer3 = 3,
    Usb = 9,
    Gpio0 = 49,
    Gpio1 = 50,
    Gpio2 = 51,
    Gpio3 = 52,
    Uart = 57,
}

impl Interrupt {
    pub const MAX: usize = 8;

    pub fn iter() -> impl Iterator<Item = Interrupt> {
        use Interrupt::*;
        [Timer1, Timer3, Usb, Gpio0, Gpio1, Gpio2, Gpio3, Uart]
            .iter()
            .map(|int| *int)
    }

    pub fn to_index(i: Interrupt) -> usize {
        use Interrupt::*;
        match i {
            Timer1 => 0,
            Timer3 => 1,
            Usb => 2,
            Gpio0 => 3,
            Gpio1 => 4,
            Gpio2 => 5,
            Gpio3 => 6,
            Uart => 7,
        }
    }

    pub fn from_index(i: usize) -> Interrupt {
        use Interrupt::*;
        match i {
            0 => Timer1,
            1 => Timer3,
            2 => Usb,
            3 => Gpio0,
            4 => Gpio1,
            5 => Gpio2,
            6 => Gpio3,
            7 => Uart,
            _ => panic!("Unknown interrupt: {}", i),
        }
    }
}

impl From<usize> for Interrupt {
    fn from(irq: usize) -> Interrupt {
        use Interrupt::*;
        match irq {
            1 => Timer1,
            3 => Timer3,
            9 => Usb,
            49 => Gpio0,
            50 => Gpio1,
            51 => Gpio2,
            52 => Gpio3,
            57 => Uart,
            _ => panic!("Unknown irq: {}", irq),
        }
    }
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    IRQ_BASIC_PENDING: ReadVolatile<u32>,   // 0x200 IRQ basic pending
    IRQ_PENDING: [ReadVolatile<u32>; 2],       // 0x204 IRQ pending 1 // 0x208 IRQ pending 2
    FIQ_CONTROL: Volatile<u32>,             // 0x20C FIQ control
    ENABLE_IRQS: [Volatile<u32>; 2],           // 0x210 Enable IRQs 1 // 0x214 Enable IRQs 2
    ENABLE_BASIC_IRQS: Volatile<u32>,       // 0x218 Enable Basic IRQs
    DISABLE_IRQS: [Volatile<u32>; 2],          // 0x21C Disable IRQs 1 // 0x220 Disable IRQs 2
    DISABLE_BASIC_IRQS: Volatile<u32>,      // 0x224 Disable Basic IRQs
}

const_assert_size!(Registers, 0x28);


/// An interrupt controller. Used to enable and disable interrupts as well as to
/// check if an interrupt is pending.
pub struct Controller {
    registers: &'static mut Registers,
}

impl Controller {
    /// Returns a new handle to the interrupt controller.
    pub fn new() -> Controller {
        Controller {
            registers: unsafe { &mut *(INT_BASE as *mut Registers) },
        }
    }

    /// Enables the interrupt `int`.
    pub fn enable(&mut self, int: Interrupt) {
        let idx = int as usize;
        self.registers.ENABLE_IRQS[idx / 32].or_mask(1 << (idx % 32));
    }

    /// Disables the interrupt `int`.
    pub fn disable(&mut self, int: Interrupt) {
        let idx = int as usize;
        self.registers.DISABLE_IRQS[idx / 32].or_mask(1 << (idx % 32));
    }

    /// Returns `true` if `int` is pending. Otherwise, returns `false`.
    pub fn is_pending(&self, int: Interrupt) -> bool {
        let idx = int as usize;
        self.registers.IRQ_PENDING[idx / 32].has_mask(1 << (idx % 32))
    }

    /// Enables the interrupt as FIQ interrupt
    pub fn enable_fiq(&mut self, int: Interrupt) {
        // Lab 5 2.B
        unimplemented!("enable_fiq")
    }
}
