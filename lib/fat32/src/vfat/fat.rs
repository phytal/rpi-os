use crate::vfat::*;
use core::fmt;
use core::ops::BitAnd;

use self::Status::*;

#[derive(Debug, PartialEq)]
pub enum Status {
    /// The FAT entry corresponds to an unused (free) cluster.
    Free,
    /// The FAT entry/cluster is reserved.
    Reserved,
    /// The FAT entry corresponds to a valid data cluster. The next cluster in
    /// the chain is `Cluster`.
    Data(Cluster),
    /// The FAT entry corresponds to a bad (disk failed) cluster.
    Bad,
    /// The FAT entry corresponds to a valid data cluster. The corresponding
    /// cluster is the last in its chain.
    Eoc(u32),
}

#[repr(C, packed)]
pub struct FatEntry(pub u32);

impl FatEntry {
    /// Returns the `Status` of the FAT entry `self`.
    pub fn status(&self) -> Status {
        let status = self.0.bitand(0x0FFFFFFF); // alignment
        match status {
            0x0000000 => Status::Free,
            0x0000001 => Status::Reserved,
            0x0000002..0xFFFFFF0 => Status::Data(Cluster::from(status)),
            0xFFFFFF0..0xFFFFFF7 => Status::Reserved,
            0xFFFFFF7 => Status::Bad,
            0xFFFFFF8..0x10000000 => Eoc(status),
            invalid => panic!("FatEntry has invalid status: {:#x}", invalid)
        }
    }
}

impl fmt::Debug for FatEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FatEntry")
            .field("value", &{ self.0 })
            .field("status", &self.status())
            .finish()
    }
}
