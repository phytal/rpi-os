use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt;
use hashbrown::HashMap;
use shim::io;

use crate::traits::BlockDevice;

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool,
}

pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// Number of sectors
    pub num_sectors: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64,
}

pub struct CachedPartition {
    device: Box<dyn BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
    partition: Partition,
}

impl CachedPartition {
    /// Creates a new `CachedPartition` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `0` will be
    /// translated to physical sector `partition.start`. Virtual sectors of
    /// sector number `[0, num_sectors)` are accessible.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T, partition: Partition) -> CachedPartition
    where
        T: BlockDevice + 'static,
    {
        assert!(partition.sector_size >= device.sector_size());

        CachedPartition {
            device: Box::new(device),
            cache: HashMap::new(),
            partition,
        }
    }

    /// Returns the number of physical sectors that corresponds to
    /// one logical sector.
    fn factor(&self) -> u64 {
        self.partition.sector_size / self.device.sector_size()
    }

    /// Maps a user's request for a sector `virt` to the physical sector.
    /// Returns `None` if the virtual sector number is out of range.
    fn virtual_to_physical(&self, virt: u64) -> Option<u64> {
        if virt >= self.partition.num_sectors {
            return None;
        }

        let physical_offset = virt * self.factor();
        let physical_sector = self.partition.start + physical_offset;

        Some(physical_sector)
    }

    /// Returns a mutable reference to the cached sector `sector`. If the sector
    /// is not already cached, the sector is first read from the disk.
    ///
    /// The sector is marked dirty as a result of calling this method as it is
    /// presumed that the sector will be written to. If this is not intended,
    /// use `get()` instead.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get_mut(&mut self, sector: u64) -> io::Result<&mut [u8]> {
        match self.cache.get_mut(&sector) {
            Some(x) => {
                (*x).dirty = true;
            }
            None => {
                let mut data = vec![0 as u8, self.partition.sector_size as u8];
                self.device.read_sector(
                    self.virtual_to_physical(sector).expect("bad caller"),
                    &mut data,
                )?;
                self.cache.insert(sector, CacheEntry { data, dirty: true });
            }
        }

        Ok(&mut self.cache.get_mut(&sector).expect("key should exist").data)
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
        match self.cache.get_mut(&sector) {
            Some(_) => {}
            None => {
                let mut data = vec![0 as u8; self.partition.sector_size as usize];
                for i in 0..self.factor() {
                    let start: usize = (i * self.device.sector_size()) as usize;
                    let end: usize = ((i + 1) * self.device.sector_size()) as usize;
                    self.device.read_sector(
                        self.virtual_to_physical(sector).expect("bad caller") + i,
                        &mut data[start..end],
                    )?;
                }
                self.cache.insert(sector, CacheEntry { data, dirty: false });
            }
        }

        Ok(&self.cache.get(&sector).expect("key should exist").data)
    }
}

// The `read_sector` and write_sector` methods should only read/write from/to cached sectors.
impl BlockDevice for CachedPartition {
    fn sector_size(&self) -> u64 {
        if self.partition.sector_size % 512 != 0 {
            panic!("Invalid CachedPartition sector size");
        }
        self.partition.sector_size
    }

    fn write_sector(&mut self, sector: u64, buf: &[u8]) -> io::Result<usize> {
        let bytes_to_write = (self.partition.sector_size as usize).min(buf.len());
        let sector = self.get_mut(sector)?;
        sector.copy_from_slice(&buf[..bytes_to_write]);
        Ok(bytes_to_write)
    }

    fn read_sector(&mut self, sector: u64, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_to_read = (self.partition.sector_size as usize).min(buf.len());
        let sector = self.get(sector)?;
        buf.copy_from_slice(&(sector[..bytes_to_read]));
        Ok(bytes_to_read)
    }
}

// why not derive debug?
impl fmt::Debug for CachedPartition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedPartition")
            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .finish()
    }
}
