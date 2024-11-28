use alloc::boxed::Box;
use alloc::vec::Vec;
use shim::io;
use shim::path::Path;

use aarch64;
use smoltcp::socket::SocketHandle;

use crate::console::kprintln;
use crate::{param::*, FILESYSTEM};
use crate::process::{Stack, State};
use crate::traps::TrapFrame;
use crate::vm::*;
// use kernel_api::{OsError, OsResult};

/// Type alias for the type of a process ID.
pub type Id = u64;

/// A structure that represents the complete state of a process.
#[derive(Debug)]
pub struct Process {
    /// The saved trap frame of a process.
    pub context: Box<TrapFrame>,
    /// The memory allocation used for the process's stack.
    pub stack: Stack,
    /// The page table describing the Virtual Memory of the process
    pub vmap: Box<UserPageTable>,
    /// The scheduling state of the process.
    pub state: State,
    // Lab 5 2.C
    //// Socket handles held by the current process
    // pub sockets: Vec<SocketHandle>,
}
use kernel_api::{OsResult, OsError};
use heap::align_down;
impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> OsResult<Process> {
        let context = Box::new(TrapFrame::default());
        let stack = Stack::new().ok_or(OsError::NoMemory)?;
        let state = State::Ready;

        let upt = UserPageTable::new();
        
        let vmap = Box::new(upt);

        let p = Process {
            context,
            stack,
            vmap,
            state
        };

        Ok(p)
    }

    /// Loads a program stored in the given path by calling `do_load()` method.
    /// Sets trapframe `context` corresponding to its page table.
    /// `sp` - the address of stack top
    /// `elr` - the address of image base.
    /// `ttbr0` - the base address of kernel page table
    /// `ttbr1` - the base address of user page table
    /// `spsr` - `F`, `A`, `D` bit should be set.
    ///
    /// Returns Os Error if do_load fails.
    pub fn load<P: AsRef<Path>>(pn: P) -> OsResult<Process> {
        use crate::VMM;

        let mut p = Process::do_load(pn)?;

        p.context.sp = Process::get_stack_top().as_u64();
        p.context.pc = Process::get_image_base().as_u64();
        p.context.ttbr0_el1 = VMM.get_baddr().as_u64();
        p.context.ttbr1_el1 = p.vmap.get_baddr().as_u64();
        use aarch64::PState;
        let mut pstate = PState::new(0);
        pstate.set_value(0b1_u64, PState::F);
        pstate.set_value(0b1_u64, PState::A);
        pstate.set_value(0b1_u64, PState::D);
        p.context.pstate = pstate.get();

        Ok(p)
    }

    /// Creates a process and open a file with given path.
    /// Allocates one page for stack with read/write permission, and N pages with read/write/execute
    /// permission to load file's contents.
    fn do_load<P: AsRef<Path>>(pn: P) -> OsResult<Process> {
        use fat32::traits::FileSystem;
        use shim::io::Read;
        let mut file = FILESYSTEM.open_file(pn)?;
        let mut p = Process::new().expect("failed to create processs");
        p.vmap.alloc(Process::get_stack_base(), PagePerm::RWX);
        
        use alloc::vec;
        let mut data = vec![];
        file.read_to_end(&mut data)?;

        let data_pages = data.chunks(PAGE_SIZE);
        let page_nums = data_pages.len();

        for (idx, data_page) in data_pages.enumerate() {
            let va = VirtualAddr::from(Process::get_image_base().as_usize()+PAGE_SIZE*idx);
            let page = p.vmap.alloc(va, PagePerm::RWX);
            page[..data_page.len()].copy_from_slice(data_page);
        }


        // alloc some pages for user heap
        // TODO: add page fault handler to automatically handle this
        let user_heap_pages = 2;
        for idx in (page_nums)..(page_nums+user_heap_pages) {
            let va = VirtualAddr::from(Process::get_image_base().as_usize()+PAGE_SIZE*idx);
            p.vmap.alloc(va, PagePerm::RWX);
        }

        Ok(p)
    }

    /// Returns the highest `VirtualAddr` that is supported by this system.
    pub fn get_max_va() -> VirtualAddr {
        let max = !0x0_u64;
        VirtualAddr::from(max)
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// memory space.
    pub fn get_image_base() -> VirtualAddr {
        VirtualAddr::from(USER_IMG_BASE)
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// process's stack.
    pub fn get_stack_base() -> VirtualAddr {
        VirtualAddr::from(align_down(Process::get_max_va().as_usize(), PAGE_SIZE))
    }

    /// Returns the `VirtualAddr` represents the top of the user process's
    /// stack.
    pub fn get_stack_top() -> VirtualAddr {
        VirtualAddr::from(align_down(Process::get_max_va().as_usize(), 0x80))
    }

    /// Returns `true` if this process is ready to be scheduled.
    ///
    /// This functions returns `true` only if one of the following holds:
    ///
    ///   * The state is currently `Ready`.
    ///
    ///   * An event being waited for has arrived.
    ///
    ///     If the process is currently waiting, the corresponding event
    ///     function is polled to determine if the event being waiting for has
    ///     occured. If it has, the state is switched to `Ready` and this
    ///     function returns `true`.
    ///
    /// Returns `false` in all other cases.
    pub fn is_ready(&mut self) -> bool {
        use core::mem::replace;
        match replace(&mut self.state, State::Dead) {
            State::Ready => {
                self.state = State::Ready;
                true
            }
            State::Waiting(mut f) => {
                let result = f(self);
                self.state = State::Waiting(f);
                result
            }
            state => {
                self.state = state;
                false
            }
        }
    }
    
}
