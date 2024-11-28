use crate::traits;
use crate::vfat::{Dir as DirStruct, File as FileStruct, Metadata, VFatHandle};

// You can change this definition if you want
#[derive(Debug)]
pub enum Entry<HANDLE: VFatHandle> {
    FileEntry(FileStruct<HANDLE>),
    DirEntry(DirStruct<HANDLE>),
}

impl<HANDLE: VFatHandle> traits::Entry for Entry<HANDLE> {
    type File = FileStruct<HANDLE>;

    type Dir = DirStruct<HANDLE>;

    type Metadata = Metadata;

    fn name(&self) -> &str {
        match &self {
            Entry::FileEntry(fe) => fe.name.as_str(),
            Entry::DirEntry(de) => de.name.as_str(),
        }
    }

    fn metadata(&self) -> &Self::Metadata {
        match &self {
            Entry::FileEntry(fe) => &fe.metadata,
            Entry::DirEntry(de) => &de.metadata.as_ref().unwrap(),
        }
    }

    fn as_file(&self) -> Option<&Self::File> {
        if let Self::FileEntry(fe) = &self {
            Some(fe)
        } else {
            None
        }
    }

    fn as_dir(&self) -> Option<&Self::Dir> {
        if let Self::DirEntry(de) = &self {
            Some(de)
        } else {
            None
        }
    }

    fn into_file(self) -> Option<Self::File> {
        if let Self::FileEntry(fe) = self {
            Some(fe)
        } else {
            None
        }
    }

    fn into_dir(self) -> Option<Self::Dir> {
        if let Self::DirEntry(de) = self {
            Some(de)
        } else {
            None
        }
    }
}

use core::fmt;
use crate::traits::{Timestamp, Entry as EntryTrait, Metadata as MetaDataTrait};
impl<HANDLE: VFatHandle> fmt::Display for Entry<HANDLE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_bool(to: &mut fmt::Formatter<'_>, b: bool, c: char) -> fmt::Result {
            if b {
                write!(to, "{}", c)
            } else {
                write!(to, "-")
            }
        }

        fn write_timestamp<T: Timestamp>(to: &mut fmt::Formatter<'_>, ts: T) -> fmt::Result {
            write!(
                to,
                "{:02}/{:02}/{} {:02}:{:02}:{:02} ",
                ts.month(),
                ts.day(),
                ts.year(),
                ts.hour(),
                ts.minute(),
                ts.second()
            )
        }

        write_bool(f, self.is_dir(), 'd')?;
        write_bool(f, self.is_file(), 'f')?;
        write_bool(f, self.metadata().read_only(), 'r')?;
        write_bool(f, self.metadata().hidden(), 'h')?;
        write!(f, "\t")?;

        write_timestamp(f, self.metadata().created())?;
        write_timestamp(f, self.metadata().modified())?;
        write_timestamp(f, self.metadata().accessed())?;
        write!(f, "\t")?;

        write!(f, "{}", self.metadata().size)?;
        write!(f, "\t")?;

        write!(f, "{}", self.name())?;
        Ok(())
    }
}
