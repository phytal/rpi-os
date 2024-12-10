use alloc::string::String;
use alloc::vec::Vec;

use shim::io::{self, SeekFrom};

use crate::traits;
use crate::vfat::{Cluster, Metadata, VFatHandle};

#[derive(Debug)]
pub struct File<HANDLE: VFatHandle> {
    pub vfat: HANDLE,           // file system handle
    pub first_cluster: Cluster, // first cluster
    pub metadata: Metadata,
    pub name : String,
    pub data : Vec<u8>,
    pub offset : usize,
}

impl<HANDLE: VFatHandle> io::Read for File<HANDLE> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_to_read = (self.data[self.offset..].len() as usize).min(buf.len());
        (buf[..bytes_to_read]).copy_from_slice(&self.data[self.offset..self.offset+bytes_to_read]);
        self.offset += bytes_to_read;
        Ok(bytes_to_read)
    }
}

impl<HANDLE: VFatHandle> io::Write for File<HANDLE> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes_to_write = (self.data[self.offset..].len() as usize).min(buf.len());
        self.data[self.offset..self.offset+bytes_to_write].copy_from_slice(&buf);
        self.offset += bytes_to_write;
        Ok(bytes_to_write)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<HANDLE: VFatHandle> io::Seek for File<HANDLE> {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        match _pos {
            SeekFrom::Start(new) => {
                if new < self.data.len() as u64 {
                    self.offset = new as usize;
                    Ok(self.offset as u64)
                } else {
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "SeekFrom::Start overflowed"))
                }
            },
            SeekFrom::End(sub) => {
                if self.data.len() > sub.abs() as usize {
                    self.offset = self.data.len() + (sub.abs() as usize);
                    Ok(self.offset as u64)
                } else {
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "SeekFrom::End overflowed"))
                }
            },
            SeekFrom::Current(add_curr) => {
                if add_curr.is_negative() && self.offset < add_curr.abs() as usize {
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "SeekFrom::Current overflowed"))
                } else {
                    self.offset = ((self.offset as i64)+add_curr) as usize;
                    Ok(self.offset as u64)
                }
            },
        }
    }
}



impl<HANDLE: VFatHandle> traits::File for File<HANDLE> {
    fn sync(&mut self) -> io::Result<()> {
        self.vfat.lock(|f| {
            f.write_chain(self.first_cluster, &self.data)
        }).map(|_| ())
    }

    fn size(&self) -> u64 {
        self.metadata.size.into()
    }
}


use alloc::fmt;
impl<HANDLE: VFatHandle> fmt::Display for File<HANDLE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // let mut buffer = Vec::new();
        // self.read_to_end(&mut buffer);
        // write!(f, "{}", buffer)?;
        match core::str::from_utf8(&self.data) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "<invalid UTF-8 data>"),
        }
    }
}