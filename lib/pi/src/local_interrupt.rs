use core::time::Duration;

use volatile::prelude::*;
use volatile::Volatile;

use aarch64::*;

const INT_BASE: usize = 0x40000000;

/// Core interrupt sources (QA7: 4.10)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LocalInterrupt {
    CntPsIrq = 0,         // CNTPSIRQ interrupt (Physical Timer -1)
    CntPnsIrq = 1,        // CNTPNSIRQ interrupt
    CntHpIrq = 2,         // CNTHPIRQ interrupt
    CntvIrq = 3,          // CNTVIRQ interrupt
    Mailbox0 = 4,         // Mailbox 0 interrupt
    Mailbox1 = 5,         // Mailbox 1 interrupt
    Mailbox2 = 6,         // Mailbox 2 interrupt
    Mailbox3 = 7,         // Mailbox 3 interrupt
    GpuInterrupt = 8,     // GPU interrupt <Can be high in one core only>
    PmuInterrupt = 9,     // PMU interrupt
    AxiOutstanding = 10,  // AXI-outstanding interrupt <For core 0 only!> all others are 0
    LocalTimer = 11,      // Local timer interrupt
}

impl LocalInterrupt {
    pub const MAX: usize = 12;

    pub fn iter() -> impl Iterator<Item = LocalInterrupt> {
        (0..LocalInterrupt::MAX).map(|n| LocalInterrupt::from(n))
    }
}

impl From<usize> for LocalInterrupt {
    fn from(irq: usize) -> LocalInterrupt {
        use LocalInterrupt::*;
        match irq { // TODO: check this
            0 => CntPsIrq,
            1 => CntPnsIrq,
            2 => CntHpIrq,
            3 => CntvIrq,
            4 => Mailbox0,
            5 => Mailbox1,
            6 => Mailbox2,
            7 => Mailbox3,
            8 => GpuInterrupt,
            9 => PmuInterrupt,
            10 => AxiOutstanding,
            11 => LocalTimer,
            _ => panic!("Unknown irq: {}", irq),
        }
    }
}

/// BCM2837 Local Peripheral Registers (QA7: Chapter 4)
#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    control_register: Volatile<u32>,
    unused_1: Volatile<u32>,
    core_timer_prescaler: Volatile<u32>,
    gpu_interrupts_routing: Volatile<u32>,
    performance_monitor_interrupts_routing_set: Volatile<u32>,
    performance_monitor_interrupts_routing_clear: Volatile<u32>,
    unused_2: Volatile<u32>,
    core_timer_access_ls: Volatile<u32>,
    core_timer_access_ms: Volatile<u32>,
    local_interrupt_routing: Volatile<u32>,
    unused_3: Volatile<u32>,
    axi_outstanding_counters: Volatile<u32>,
    axi_outstanding_irq: Volatile<u32>,
    local_timer_control_status: Volatile<u32>,
    local_timer_write_flags: Volatile<u32>,
    unused_4: Volatile<u32>,
    core_timers_interrupt_control: [Volatile<u32>; 4],
    core_mailboxes_interrupt_control: [Volatile<u32>; 4],
    core_irq_source: [Volatile<u32>; 4],
    core_fiq_source: [Volatile<u32>; 4],
    core_mailbox_write_set: [[Volatile<u32>; 4]; 4],
    core_mailbox_read_write_high_to_clear: [[Volatile<u32>; 4]; 4],
}

shim::const_assert_size!(Registers, 0x100);

pub struct LocalController {
    core: usize,
    registers: &'static mut Registers,
}

impl LocalController {
    /// Returns a new handle to the interrupt controller.
    pub fn new(core: usize) -> LocalController {
        LocalController {
            core: core,
            registers: unsafe { &mut *(INT_BASE as *mut Registers) },
        }
    }

    pub fn enable_local_timer(&mut self) {
        // Lab 5 1.C
        unsafe {
            // To generate an interrupt, software must set ENABLE to 1 and clear IMASK to 0.
            // ENABLE, bit [0] 
            // IMASK, bit [1]
            CNTP_CTL_EL0.set(CNTP_CTL_EL0::ENABLE);
        }
        // nCNTPNSIRQ IRQ control
        self.registers.core_timers_interrupt_control[self.core].write(0x2);
    }

    pub fn is_pending(&self, int: LocalInterrupt) -> bool {
        // Lab 5 1.C
        let v = int as isize;
        self.registers.core_irq_source[self.core].has_mask(0x1 << v)
    }

    pub fn tick_in(&mut self, t: Duration) {
        // Lab 5 1.C
        unsafe {
            let duration = (t.as_micros() as u64 * CNTFRQ_EL0.get()) / 1_000_000 as u64;
            // clear via writing to TVAL or CVAL, so that firing condition is no longer met
            CNTP_TVAL_EL0.set(CNTP_TVAL_EL0::TVAL & duration);
        }
    }
}

pub fn local_tick_in(core: usize, t: Duration) {
    LocalController::new(core).tick_in(t);
}
