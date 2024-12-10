use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;
use alloc::string::String;

use shim::io;
use shim::path::Path;

use crate::mbr::MasterBootRecord;
use crate::traits::{BlockDevice, FileSystem};
use crate::util::SliceExt;
use crate::vfat::{BiosParameterBlock, CachedPartition, Partition};
use crate::vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Status};

/// A generic trait that handles a critical section as a closure
pub trait VFatHandle: Clone + Debug + Send + Sync {
    fn new(val: VFat<Self>) -> Self;
    fn lock<R>(&self, f: impl FnOnce(&mut VFat<Self>) -> R) -> R;
}

#[derive(Debug)]
pub struct VFat<HANDLE: VFatHandle> {
    phantom: PhantomData<HANDLE>,
    device: CachedPartition,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    _sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    rootdir_cluster: Cluster,
}

impl<HANDLE: VFatHandle> VFat<HANDLE> {
    pub fn from<T>(mut device: T) -> Result<HANDLE, Error>
    where
        T: BlockDevice + 'static,
    {
        let mbr = MasterBootRecord::from(&mut device)?;
        let p_entry = mbr.get_fat32_partition()?;
        let bpb = BiosParameterBlock::from(&mut device, p_entry.relative_sector as u64)?;
        let partition = Partition {
            start: p_entry.relative_sector as u64,
            num_sectors: p_entry.total_sectors as u64,
            sector_size: bpb.bytes_per_sector as u64,
        };

        let cp = CachedPartition::new(device, partition);

        let dss: u64 =
            (bpb.num_reserved_sectors as u64) + (bpb.num_fats as u32 * bpb.sectors_per_fat) as u64;
        let vfat: VFat<HANDLE> = VFat {
            phantom: PhantomData,
            device: cp,
            bytes_per_sector: bpb.bytes_per_sector,
            sectors_per_cluster: bpb.sectors_per_cluster,
            _sectors_per_fat: bpb.sectors_per_fat,
            fat_start_sector: bpb.num_reserved_sectors as u64,
            data_start_sector: dss,
            rootdir_cluster: Cluster::from(bpb.root_dir_cluster),
        };
        Ok(HANDLE::new(vfat))
    }

    // Recommended
    fn cluster_start_sector(&mut self, cluster: Cluster) -> io::Result<u64> {
        let cluster_num: u32 = cluster.into();
        let sfc =
            self.data_start_sector + (((cluster_num - 2) as u64) * self.sectors_per_cluster as u64); // sub 2 because cluster 2 is at data_start_sector
                                                                                                     // add error checking (return a sector out of range)
        Ok(sfc)
    }

    //
    //  * A method to read from an offset of a cluster into a buffer.
    //
    fn read_cluster(
        &mut self,
        cluster: Cluster,
        offset: usize,
        buf: &mut Vec<u8>,
    ) -> io::Result<usize> {
        let curr_sector =
            self.cluster_start_sector(cluster)? + offset as u64 / self.bytes_per_sector as u64;
        let mut bytes_read = 0;
        for i in 0..self.sectors_per_cluster {
            bytes_read += self
                .device
                .read_all_sector(curr_sector+i as u64, buf)?;
        }
        Ok(bytes_read)
    }
    fn write_cluster(&mut self, cluster: Cluster, offset: usize, buf: &[u8]) -> io::Result<usize> {
        let mut curr_sector =
            self.cluster_start_sector(cluster)? + offset as u64 / self.bytes_per_sector as u64;
        let mut curr_offset = offset % (self.bytes_per_sector as usize);

        let mut bytes_read = 0;
        for _ in 0..self.sectors_per_cluster {
            bytes_read += self.device.write_sector(curr_sector, &buf[curr_offset..])?;

            curr_offset += bytes_read;
            curr_sector += 1;
        }
        Ok(bytes_read)
    }
    //
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    //
    pub fn read_chain(&mut self, start: Cluster, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut curr_cluster = start;
        let mut bytes_read = 0;

        loop {
            bytes_read += self.read_cluster(curr_cluster, 0, buf)?;
            match self.fat_entry(curr_cluster)?.status() {
                Status::Data(cluster) => {
                    curr_cluster = cluster;
                },
                Status::Eoc(_) => {
                    break;
                }
                Status::Free => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Writing: found cluster with free status",
                    ));
                },
                Status::Reserved => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Reading: found cluster with res status",
                    ));
                },
                Status::Bad => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Reading: found cluster with bad status",
                    ));
                }
            }
        }
        Ok(bytes_read)
    }
    pub fn write_chain(&mut self, start: Cluster, buf: &Vec<u8>) -> io::Result<usize> {
        let mut curr_cluster = start;
        let mut bytes_write = 0;

        loop {
            bytes_write += self.write_cluster(curr_cluster, 0, buf)?;

            match self.fat_entry(curr_cluster)?.status() {
                Status::Data(cluster) => {
                    curr_cluster = cluster;
                },
                Status::Eoc(_) => {
                    break;
                }
                Status::Free => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Writing: found cluster with free status",
                    ));
                },
                Status::Reserved => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Writing: found cluster with res status",
                    ));
                },
                Status::Bad => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Writing: found cluster with bad status",
                    ));
                }
            }
        }
        Ok(bytes_write)
    }
    //
    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        

        let entries_per_sector = self.bytes_per_sector / size_of::<FatEntry>() as u16;
        let fat_sector: u64 = self.fat_start_sector + (cluster.num() as u64 / entries_per_sector as u64);
        let buf = self.device.get(fat_sector)?;
        let entries: &[FatEntry] = unsafe { buf.cast() };
        
        Ok(&entries[(cluster.num() as usize) % entries_per_sector as usize])
    }

    pub fn get_root_dir(&mut self, handle: &HANDLE) -> io::Result<Dir<HANDLE>> {
        Ok(Dir {
            first_cluster: self.rootdir_cluster,
            vfat: handle.clone(),
            name: String::from("/"),
            metadata: None,
        })
    }
}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Entry = Entry<HANDLE>;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        let root_dir = self.lock(|s| s.get_root_dir(&self).unwrap());
        root_dir.open_path(path.as_ref(), &mut Path::new("/").to_path_buf())
    }
}
