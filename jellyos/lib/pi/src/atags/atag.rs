use crate::atags::raw;

pub use crate::atags::raw::{Core, Mem};

/// An ATAG.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Atag {
    Core(raw::Core),
    Mem(raw::Mem),
    Cmd(&'static str),
    Unknown(u32),
    None,
}

impl Atag {
    /// Returns `Some` if this is a `Core` ATAG. Otherwise returns `None`.
    pub fn core(self) -> Option<Core> {
        if let Atag::Core(core) = self {
            Some(core)
        } else {
            None
        }
    }

    /// Returns `Some` if this is a `Mem` ATAG. Otherwise returns `None`.
    pub fn mem(self) -> Option<Mem> {
        if let Atag::Mem(mem) = self {
            Some(mem)
        } else {
            None
        }
    }

    /// Returns `Some` with the command line string if this is a `Cmd` ATAG.
    /// Otherwise returns `None`.
    pub fn cmd(self) -> Option<&'static str> {
        if let Atag::Cmd(cmd) = self {
            Some(cmd)
        } else {
            None
        }
    }
}

use core::{slice::from_raw_parts, str};
impl From<&'static raw::Atag> for Atag {
    fn from(atag: &'static raw::Atag) -> Atag {
        unsafe {
            match (atag.tag, &atag.kind) {
                (raw::Atag::CORE, &raw::Kind { core }) => Atag::Core(core),
                (raw::Atag::MEM, &raw::Kind { mem }) => Atag::Mem(mem),
                (raw::Atag::CMDLINE, &raw::Kind { ref cmd }) => {
                    let start = &cmd.cmd as *const u8;
                    let mut length : usize = 0;
                    while *(start.add(length)) != b'\0' {length += 1;}
                    let xs = from_raw_parts(start, length);
                    Atag::Cmd(str::from_utf8_unchecked(xs))
                },
                (raw::Atag::NONE, _) => Atag::None,
                (id, _) => Atag::Unknown(id),
            }
        }
    }
}
