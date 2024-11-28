use core::char::decode_utf16;
use core::ops::BitAnd;

use alloc::string::String;
use alloc::vec::Vec;

use hashbrown::HashMap;
use shim::const_assert_size;
use shim::ffi::OsStr;
use shim::io;
use shim::path::PathBuf;

use crate::traits;
use crate::traits::Dir as DirTrait;
use crate::traits::Entry as EntryTrait;
use crate::util::VecExt;
use crate::vfat::{Attributes, Date, Metadata, Time};
use crate::vfat::{Cluster, Entry, File, VFatHandle};

#[derive(Debug, Clone)]
pub struct Dir<HANDLE: VFatHandle> {
    pub vfat: HANDLE,           // file system handle
    pub first_cluster: Cluster, // first cluster
    pub name: String,
    pub metadata: Option<Metadata>,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    file_name: [u8; 8],
    file_extension: [u8; 3],
    file_attributes: Attributes,
    reserved_win: u8,
    creation_secs_tenths: u8,
    creation_time: Time,
    creation_date: Date,
    accessed_date: Date,
    high_cluster_num: u16,
    modification_time: Time,
    modification_date: Date,
    low_cluster_num: u16,
    file_size: u32,
}

const_assert_size!(VFatRegularDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
/// Long file name (LFN) entries were added to FAT32 to allow for filenames greater than 11 characters in length.
/// If an entry has a name greater than 11 characters in length, then its regular directory entry is preceded by as many LFN entries as needed to store the bytes for the entry’s name.
pub struct VFatLfnDirEntry {
    /// LFN entries are not ordered physically. Instead, they contain a field that indicates their sequence.
    sequence_num: u8,
    first_name_chars: [u16; 5],
    file_attributes: Attributes,
    file_type: u8,
    checksum: u8,
    second_name_chars: [u16; 6],
    zeroes: u16,
    third_name_chars: [u16; 2],
}

const_assert_size!(VFatLfnDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    /// ID of 0x00. Indicates the end of the directory.
    /// ID of 0xE5: Marks an unused/deleted entry.
    /// All other IDs make up part of the file’s name or LFN sequence number.
    file_id: u8,
    _reserved: [u8; 10],
    /// The byte at offset 11 determines whether the entry is a regular entry or an LFN entry.
    /// Value of 0x0F: entry is an LFN entry.
    /// All other values: entry is a regular entry.
    reg_or_lfn: u8,
    _reserved2: [u8; 20],
}

const_assert_size!(VFatUnknownDirEntry, 32);

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}
use shim::path;
impl<HANDLE: VFatHandle> Dir<HANDLE> {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry<HANDLE>> {
        for entry in self.entries()? {
            if entry.name().eq_ignore_ascii_case(
                name.as_ref()
                    .to_str()
                    .expect("failed to get str from osstr"),
            ) {
                if let Some(entry_dir) = entry.as_dir() {
                    if entry_dir.first_cluster.num() == 0 {
                        return Ok(Entry::DirEntry(self.vfat.lock(|s| s.get_root_dir(&self.vfat).unwrap())));
                    }
                }
                return Ok(entry);
            }
        }
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no entry with that name",
        ));
    }

    pub fn open_path<P: AsRef<path::Path>>(&self, path: P, pwd: &mut PathBuf) -> io::Result<Entry<HANDLE>> {
        let mut curr = Entry::DirEntry(self.clone());
        let mut working_pwd = pwd.clone();
        for component in path.as_ref().components() {
            match component {
                // path::Component::Prefix(_) => todo!(),
                path::Component::RootDir => {
                    curr = Entry::DirEntry(self.vfat.lock(|s| s.get_root_dir(&self.vfat.clone()))?);
                    working_pwd = path::Path::new("/").to_path_buf();
                },
                path::Component::CurDir => {
                    continue;
                },
                path::Component::ParentDir => {
                    if let Some(dir) = curr.as_dir() {
                        curr = dir.find("..")?;
                        
                        working_pwd.pop();
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "No parent dir",
                        ));
                    }
                },
                path::Component::Normal(name) => {
                    if let Some(dir) = curr.as_dir() {
                        curr = dir.find(name)?;
                        working_pwd = working_pwd.join(name);
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "Path not found",
                        ));
                    }
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "Path not found",
                    ))
                }
            }
        }
        *pwd = working_pwd;
        Ok(curr)
    }
}

pub struct DirIterator<HANDLE: VFatHandle> {
    directory_data: Vec<VFatDirEntry>,
    index: usize,
    vfat: HANDLE,
}

// this is really something
impl<HANDLE: VFatHandle> Iterator for DirIterator<HANDLE> {
    type Item = Entry<HANDLE>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.directory_data.len() {
            return None;
        }

        let mut name: String = String::new();
        let mut lfn_data: HashMap<u8, Vec<u16>> = HashMap::new();
        let regular_entry: VFatRegularDirEntry;
        let mut counter: i32 = 0;
        loop {
            let unknown_entry: VFatUnknownDirEntry =
                unsafe { self.directory_data[self.index].unknown };

            if unknown_entry.file_id == 0xE5 {
                self.index += 1;
                continue;
            } else if unknown_entry.reg_or_lfn.bitand(0x0F) == 0x0F {
                let mut local_data: Vec<u16> = Vec::new();
                let lfn_entry: VFatLfnDirEntry =
                    unsafe { self.directory_data[self.index].long_filename };
                let idx = lfn_entry.sequence_num & 0xF;
                let temp_copy1 = lfn_entry.first_name_chars;
                let temp_copy2 = lfn_entry.second_name_chars;
                let temp_copy3 = lfn_entry.third_name_chars;
                let char_sets: Vec<&[u16]> = vec![&temp_copy1, &temp_copy2, &temp_copy3];
                'outer: for char_set in char_sets {
                    for ch in char_set {
                        if [0x0000, 0xFFFF].contains(&ch) {
                            break 'outer;
                        }
                        local_data.push(*ch);
                    }
                }
                lfn_data.insert(idx, local_data);
                self.index += 1;
                if lfn_entry.sequence_num.bitand(0x40) == 0x40 {
                    counter -= idx as i32;
                } else {
                    counter += 1;
                }
                if counter == -1 {
                    // math
                    assert!(name.is_empty());
                    let mut p = lfn_data.len();
                    p += 1;

                    for key in 1..p {
                        if let Some(data) = lfn_data.get(&(key as u8)) {
                            for utf16_char in decode_utf16(data.clone()) {
                                name.push(utf16_char.ok()?);
                            }
                        } else {
                            return None; // error
                        }
                    }
                }
            } else if unknown_entry.file_id == 0x00 {
                return None;
            } else {
                // When parsing a directory entry’s name, you must manually add a . to the non-LFN based directory entries to demarcate the file’s extension.
                // You should only add a . if the file’s extension is non-empty
                // You should only add a . if the file’s extension is non-empty
                regular_entry = unsafe { self.directory_data[self.index].regular };
                self.index += 1;
                if !name.is_empty() {
                    break;
                } // assigned as LFN. can do checksum here as well
                let bad = [0x00, 0x20];
                for ch in regular_entry.file_name {
                    if bad.contains(&ch) {
                        break;
                    }
                    name.push(ch.into());
                }
                let bad = [0x20, 0x20, 0x20];
                if regular_entry.file_extension != bad {
                    name.push('.');
                    for ch in regular_entry.file_extension {
                        if [0x00, 0x20].contains(&ch) {
                            break;
                        }
                        name.push(ch.into());
                    }
                }
                break;
            }
        }

        // process regular entry
        // get metadata
        let metadata = Metadata {
            attributes: regular_entry.file_attributes,
            created_time: regular_entry.creation_time,
            created_date: regular_entry.creation_date,
            accessed_date: regular_entry.accessed_date,
            modified_time: regular_entry.modification_time,
            modified_date: regular_entry.modification_date,
            size: regular_entry.file_size
        };

        // get first_cluster
        let first_cluster: u32 =
            regular_entry.low_cluster_num as u32 | ((regular_entry.high_cluster_num as u32) << 16);

        if regular_entry.file_attributes.0.bitand(0x10) == 0x10 {
            // directory
            return Some(Entry::DirEntry(Dir {
                first_cluster: first_cluster.into(),
                vfat: self.vfat.clone(),
                metadata: Some(metadata),
                name,
            }));
        } else {
            // file
            let mut data: Vec<u8> = Vec::new();
            if first_cluster != 0 {
                // Volume Label
                let br = self.vfat
                    .lock(|s| s.read_chain(first_cluster.into(), &mut data))
                    .ok()?;
                let j = regular_entry.file_size;
                assert!(br >= j as usize, "br = {}, regular_entry.file_size = {}", br, j);
            }
            return Some(Entry::FileEntry(File {
                first_cluster: first_cluster.into(),
                vfat: self.vfat.clone(),
                metadata,
                name,
                data: data[..(regular_entry.file_size as usize)].to_vec(),
                offset: 0,
            }));
        }
    }
}
impl<HANDLE: VFatHandle> traits::Dir for Dir<HANDLE> {
    type Entry = Entry<HANDLE>;
    type Iter = DirIterator<HANDLE>;

    /// . You will likely need to use at-most one line of unsafe when implementing entries();
    /// you may find the VecExt and SliceExt trait implementations we have provided particularly useful here.
    fn entries(&self) -> io::Result<Self::Iter> {
        let mut data: Vec<u8> = Vec::new();
        // TODO: why are we reading an entire chain into memory? we should only do this on demand.
        self.vfat
            .lock(|s| s.read_chain(self.first_cluster, &mut data))?; //
        Ok(DirIterator {
            directory_data: unsafe { data.cast::<VFatDirEntry>() },
            index: 0,
            vfat: self.vfat.clone(),
        })
    }
}
