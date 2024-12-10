use shim::const_assert_size;
use shim::io;

use crate::traits::BlockDevice;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CHS {
    head: u8,
    sector_and_cylinder: [u8; 2],
}

impl Default for CHS {
    fn default() -> CHS {
        CHS {
            head: 0,
            sector_and_cylinder: [0; 2],
        }
    }
}

const_assert_size!(CHS, 3);

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct PartitionEntry {
    boot_indicator: u8,
    start_chs: CHS,
    partition_type: u8,
    end_chs: CHS,
    pub relative_sector: u32,
    pub total_sectors: u32,
}

impl Default for PartitionEntry {
    fn default() -> PartitionEntry {
        PartitionEntry {
            boot_indicator: 0,
            start_chs: CHS::default(),
            partition_type: 0,
            end_chs: CHS::default(),
            relative_sector: 0,
            total_sectors: 0,
        }
    }
}

const_assert_size!(PartitionEntry, 16);

/// The master boot record (MBR).
#[repr(C, packed)]
#[derive(Debug)]
pub struct MasterBootRecord {
    pub mbr_boostrap: [u8; 436],
    pub disk_id: [u8; 10],
    pub partition_table: [PartitionEntry; 4],
    pub signature: [u8; 2],
}

impl Default for MasterBootRecord {
    fn default() -> MasterBootRecord {
        MasterBootRecord {
            mbr_boostrap: [0; 436],
            disk_id: [0; 10],
            partition_table: [PartitionEntry::default(); 4],
            signature: [0; 2],
        }
    }
}

const_assert_size!(MasterBootRecord, 512);

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

const VALID_SIGNATURE: [u8; 2] = [0x55, 0xAA];
const VALID_INDICATORS: [u8; 2] = [0x80, 0x00];

use core::slice;
impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut mbr = MasterBootRecord::default();
        let mbr_slice: &mut [u8] = unsafe {
            slice::from_raw_parts_mut(
                &mut mbr as *mut MasterBootRecord as *mut u8,
                size_of::<MasterBootRecord>(),
            )
        };
        device
            .read_sector(0, mbr_slice)
            .map_err(|err| Error::Io(err))?;
        if mbr.signature != VALID_SIGNATURE {
            return Err(Error::BadSignature);
        }
        for (i, partition) in mbr.partition_table.iter().enumerate() {
            if !VALID_INDICATORS.contains(&partition.boot_indicator) {
                return Err(Error::UnknownBootIndicator(i as u8));
            }
        }

        Ok(mbr)
    }

    /// Assuming we get the first valid FAT32 partition here!
    pub fn get_fat32_partition(&self) -> Result<PartitionEntry, Error> {
        if let Some(p) = self
            .partition_table
            .iter()
            .find(|p| [0xB, 0xC].contains(&p.partition_type))
        {
            Ok(*p)
        } else {
            Err(Error::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "No valid FAT32 partition",
            )))
        }
    }
}
