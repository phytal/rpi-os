mod frame;
mod syndrome;
mod syscall;

pub mod irq;
use crate::console::{kprint, kprintln};

pub use self::frame::TrapFrame;

use crate::GLOBAL_IRQ;
use aarch64::{affinity, current_el, FAR_EL1};
use pi::interrupt::{Controller, Interrupt};
use pi::local_interrupt::{LocalController, LocalInterrupt};

use self::syndrome::Syndrome;
use self::syscall::handle_syscall;
use crate::percore;
use crate::traps::irq::IrqHandlerRegistry;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}
use crate::{shell, IRQ};
/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern "C" fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    //kprintln!("info: {:#?}", info);
    match info.kind {
        Kind::Synchronous if let Syndrome::Svc(num) = Syndrome::from(esr) => {
            // kprintln!("tf: {:#?}", tf);
            handle_syscall(num, tf);
        }
        Kind::Synchronous => {
            match Syndrome::from(esr) {
                Syndrome::DataAbort { kind, level } => {
                    unsafe {
                        kprintln!("Fault addr: {:x}", FAR_EL1.get());
                    }
                },
                _ => {}
            }
            // Preferred Exception Return Address for synchronous
            // is the address of instr that generated exception
            panic!("{:#?}, {}, {:#?}", info, esr, Syndrome::from(esr));
            tf.pc += 4;
        }
        Kind::Irq => {
            let mut handled = false;

            let global_controller = Controller::new();
            if affinity() == 0 {
                for i in Interrupt::iter() {
                    if global_controller.is_pending(i) {
                        // kprintln!("{:#?}, idx:{:#?} ", info, i);
                        GLOBAL_IRQ.invoke(i, tf);
                        handled = true;
                        break;
                    }
                }
            }

            if !handled {
                let local_controller = LocalController::new(affinity());
                for i in LocalInterrupt::iter() {
                    if local_controller.is_pending(i) {
                        // kprintln!("{:#?}, idx:{:#?} ", info, i);
                        IRQ.invoke(i, tf);
                        handled = true;
                        break;
                    }
                }
            }

            if !handled {
                panic!("interrupt not handled");
            }
        }
        Kind::Fiq => {
            kprintln!("FIQ trap");
        }
        Kind::SError => {
            kprintln!("SError trap");
        }
    }
}
