use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use pi::interrupt;

use core::borrow::Borrow;
use core::ffi::c_void;
use core::fmt;
use core::mem;
use core::time::Duration;
use core::u64;

use aarch64::*;
use pi::local_interrupt::LocalInterrupt;
use smoltcp::time::Instant;

use crate::console::kprint;
use crate::console::kprintln;
use crate::mutex::Mutex;
use crate::net::uspi::TKernelTimerHandle;
use crate::param::*;
use crate::percore::{get_preemptive_counter, is_mmu_ready, local_irq};
use crate::process::{Id, Process, State};
use crate::traps::irq::IrqHandlerRegistry;
use crate::traps::TrapFrame;
use crate::vm::UserPageTable;
use crate::GLOBAL_IRQ;
use crate::{ETHERNET, USB};

/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Box<Scheduler>>>);


impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> GlobalScheduler {
        GlobalScheduler(Mutex::new(None))
    }

    /// Enters a critical region and execute the provided closure with a mutable
    /// reference to the inner scheduler.
    pub fn critical<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Scheduler) -> R,
    {
        let mut guard = self.0.lock();
        f(guard.as_mut().expect("scheduler uninitialized"))
    }

    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, process: Process) -> Option<Id> {
        self.critical(move |scheduler| scheduler.add(process))
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::schedule_out()` and `Scheduler::switch_to()`.
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Id {
        let mut old_tf = tf.clone();
        let id = self.switch_to(tf);
        if id != u64::MAX {
            self.critical(|scheduler| {
                scheduler.schedule_out(new_state, &mut old_tf);
            });
        }
        id
    }

    /// Edited to fix deadlock
    /// For more details, see the documentation on `Scheduler::switch_to()`.
    ///
    /// Returns the process's ID when a ready process is found.
    pub fn switch_to(&self, tf: &mut TrapFrame) -> Id {
        let rtn = self.critical(|scheduler| scheduler.switch_to(tf));
        if let Some(id) = rtn {
            trace!(
                "[core-{}] switch_to {:?}, pc: {:x}, lr: {:x}, x29: {:x}, x28: {:x}, x27: {:x}",
                affinity(),
                id,
                tf.pc,
                tf.regs[30],
                tf.regs[29],
                tf.regs[28],
                tf.regs[27]
            );
            return id;
        } else {
            return u64::MAX;
        }
            // problem: when a process ends or hasn't started (idle cores), they hold the IRQ lock
            // causes live threads to not make progress since they get deadlocked on invoking the
            // IRQ function
            //aarch64::wfi();
    }

    /// Kills currently running process and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::kill()`.
    #[must_use]
    pub fn kill(&self, tf: &mut TrapFrame) -> Option<Id> {
        self.critical(|scheduler| scheduler.kill(tf))
    }

    /// Starts executing processes in user space using timer interrupt based
    /// preemptive scheduling. This method should not return under normal conditions.
    pub fn start(&'static self) -> ! {
        // register handler fn for timer
        if aarch64::affinity() == 0 {
            self.initialize_global_timer_interrupt();
        }
        self.initialize_local_timer_interrupt();


        let mut tf = Box::new(TrapFrame::default());
        tf.sp = Process::get_stack_top().as_u64();
        tf.pc = Process::get_image_base().as_u64();
        let mut pstate = PState::new(0);
        pstate.set_value(0b1_u64, PState::F);
        pstate.set_value(0b1_u64, PState::A);
        pstate.set_value(0b1_u64, PState::D);
        tf.pstate = pstate.get();
        tf.tpidr = u64::MAX;
        tf.ttbr0_el1 = crate::VMM.get_baddr().as_u64();
        let upt = crate::vm::UserPageTable::new();
        let mut vmap = Box::new(upt);
        tf.ttbr1_el1 = vmap.get_baddr().as_u64();
        self.load_code(&mut vmap, idle_proc as *const u8);
        let frame_addr = tf.as_ref() as *const TrapFrame as *const u64 as u64;

        unsafe {
            asm!(
                "mov x0, {context:x}",
                "bl idle_context_restore",
                "eret",
                context = in(reg) frame_addr,
            );
        }
        loop {}
    }

    /// # Lab 4
    /// Initializes the global timer interrupt with `pi::timer`. The timer
    /// should be configured in a way that `Timer1` interrupt fires every
    /// `TICK` duration, which is defined in `param.rs`.
    ///
    /// # Lab 5
    /// Registers a timer handler with `Usb::start_kernel_timer` which will
    /// invoke `poll_ethernet` after 1 second.
    pub fn initialize_global_timer_interrupt(&'static self) {
    }

    /// Initializes the per-core local timer interrupt with `pi::local_interrupt`.
    /// The timer should be configured in a way that `CntpnsIrq` interrupt fires
    /// every `TICK` duration, which is defined in `param.rs`.
    pub fn initialize_local_timer_interrupt(&'static self) {
        // Lab 5 2.C
        use crate::IRQ;
        use pi::interrupt::Interrupt;
        IRQ.register(
            LocalInterrupt::CntPnsIrq,
            Box::new(|tf: &mut TrapFrame| {
                // tf was the interrupted processes' trap frame
                pi::local_interrupt::local_tick_in(aarch64::affinity(), crate::param::TICK);
                self.switch(State::Ready, tf); // context switch

                // kprintln!("interrupt");
                // let binding = self.0.lock();
                // let t = binding.as_ref().unwrap();
                // for p in &t.processes {
                //     kprintln!("{:#?}", p.state);
                // }
                // kprintln!("");

                // if let Some(id) = self.kill(tf) {
                //     kprintln!("{}", id);
                // }
            }),
        );

        // enable timer interrupts
        pi::local_interrupt::LocalController::new(aarch64::affinity()).enable_local_timer();

        // set timer interupt
        pi::local_interrupt::local_tick_in(aarch64::affinity(), crate::param::TICK);
    }

    /// Initializes the scheduler and add userspace processes to the Scheduler.
    pub unsafe fn initialize(&self) {
        *self.0.lock() = Some(Box::new(Scheduler::new()));

         for _ in 0..NCORES*2 {
         //for _ in 0..1 {
            use shim::path::Path;
            let p = Process::load(Path::new("/programs/fib.bin")).expect("failed to load fib proc");
            self.add(p);
        }
    }

    // The following method may be useful for testing Lab 4 Phase 3:
    //
    // * A method to load a extern function to the user process's page table.
    //
    pub fn load_code(&self, vmap: &mut Box<UserPageTable>, f: *const u8) {
        use crate::vm::{PagePerm, VirtualAddr};

        let page: &mut [u8] = vmap
            .alloc(VirtualAddr::from(USER_IMG_BASE as u64), PagePerm::RWX);

        let text: &[u8] = unsafe { core::slice::from_raw_parts(f, 100) };

        page[0..100].copy_from_slice(text);
    }
}

/// Poll the ethernet driver and re-register a timer handler using
/// `Usb::start_kernel_timer`.
extern "C" fn poll_ethernet(_: TKernelTimerHandle, _: *mut c_void, _: *mut c_void) {
    // Lab 5 2.B
    unimplemented!("poll_ethernet")
}

/// Internal scheduler struct which is not thread-safe.
pub struct Scheduler {
    processes: VecDeque<Process>, // queue
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Scheduler {
        Scheduler {
            processes: VecDeque::new(),
            last_id: None,
        }
    }

    /// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
        let new_id = self.processes.len() as u64;
        //kprint!("{}", self.processes.len());

        process.context.tpidr = new_id;
        self.processes.push_back(process);
        Some(new_id)
    }

    /// Finds the currently running process, sets the current process's state
    /// to `new_state`, prepares the context switch on `tf` by saving `tf`
    /// into the current process, and push the current process back to the
    /// end of `processes` queue.
    ///
    /// If the `processes` queue is empty or there is no current process,
    /// returns `false`. Otherwise, returns `true`.
    fn schedule_out(&mut self, new_state: State, tf: &mut TrapFrame) -> bool {
        for (i, p) in self.processes.iter_mut().enumerate() {
            if matches!(p.state, State::Running) && p.context.tpidr == tf.tpidr {
                p.state = new_state;
                p.context = Box::new(*tf);
                let rproc = self.processes.remove(i).unwrap();
                self.processes.push_back(rproc);
                return true;
            }
        }
        false
    }

    /// Finds the next process to switch to, brings the next process to the
    /// front of the `processes` queue, changes the next process's state to
    /// `Running`, and performs context switch by restoring the next process`s
    /// trap frame into `tf`.
    ///
    /// If there is no process to switch to, returns `None`. Otherwise, returns
    /// `Some` of the next process`s process ID.
    fn switch_to(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        for (i, p) in self.processes.iter_mut().enumerate() {
            if p.is_ready() {
                p.state = State::Running;
                let rproc = self.processes.remove(i).unwrap();
                let pid = rproc.context.tpidr;

                *tf = *rproc.context; // context switch bro
                self.processes.push_front(rproc);

                self.last_id = Some(pid);
                return Some(pid);
            }
        }
        None
    }

    /// Kills currently running process by scheduling out the current process
    /// as `Dead` state. Releases all process resources held by the process,
    /// removes the dead process from the queue, drops the dead process's
    /// instance, and returns the dead process's process ID.
    fn kill(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        for (i, p) in self.processes.iter_mut().enumerate() {
            if matches!(p.state, State::Running) && p.context.tpidr == tf.tpidr {
                p.state = State::Dead;
                let rproc = self.processes.remove(i).unwrap();
                let pid = rproc.context.tpidr;
                drop(rproc); // Explicitly drop the process instance
                return Some(pid);
            }
        }
        None
    }

    /// Releases all process resources held by the current process such as sockets.
    fn release_process_resources(&mut self, tf: &mut TrapFrame) {
        // Lab 5 2.C
        unimplemented!("release_process_resources")
    }

    /// Finds a process corresponding with tpidr saved in a trap frame.
    /// Panics if the search fails.
    pub fn find_process(&mut self, tf: &TrapFrame) -> Option<&mut Process> {
        for i in 0..self.processes.len() {
            if self.processes[i].context.tpidr == tf.tpidr {
                return Some(&mut self.processes[i]);
            }
        }
        None
    }
}

impl fmt::Debug for Scheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.processes.len();
        write!(f, "  [Scheduler] {} processes in the queue\n", len)?;
        for i in 0..len {
            write!(
                f,
                "    queue[{}]: proc({:3})-{:?} \n",
                i, self.processes[i].context.tpidr, self.processes[i].state
            )?;
        }
        Ok(())
    }
}

pub extern "C" fn test_user_process() -> ! {
    loop {
        let ms = 10000;
        let error: u64;
        let elapsed_ms: u64;

        unsafe {
            asm!(
                "mov x0, {ms:x}",
                "svc {svc_num}",
                "mov {ems}, x0",
                "mov {error}, x7",
                ms = in(reg) ms,
                svc_num = const 1,
                ems = out(reg) elapsed_ms,
                error = out(reg) error,
                out("x0") _,   // Clobbers x0
                out("x7") _,   // Clobbers x7
                options(nostack),
            );
        }
    }
}

use core::arch::asm;

use crate::shell;

extern "C" fn idle_proc() {
    loop {
        // kprintln!("idle proc here");
        // timer::spin_sleep(Duration::from_secs(1));
    }
}

