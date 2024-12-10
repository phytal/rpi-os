use core::fmt;
use core::ops::BitAnd;

use crate::traits::{self, Timestamp as TimestampTrait, Metadata as MetadataTrait};


/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(pub u8);

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub date: Date,
    pub time: Time,
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    pub attributes: Attributes,
    pub created_time: Time,
    pub created_date: Date,
    pub accessed_date: Date,
    pub modified_time: Time,
    pub modified_date: Date,
    pub size : u32
}

impl traits::Timestamp for Timestamp {
    fn year(&self) -> usize {
        // Bits 15-9 represent the year offset from 1980
        let val = (self.date.0 >> 9) & 0x7F;
        (val as usize) + 1980
    }

    fn month(&self) -> u8 {
        // Bits 8-5 represent the month
        ((self.date.0 >> 5) & 0xF) as u8
    }

    fn day(&self) -> u8 {
        // Bits 4-0 represent the day
        (self.date.0 & 0x1F) as u8
    }

    fn hour(&self) -> u8 {
        // Bits 15-11 represent the hour
        ((self.time.0 >> 11) & 0x1F) as u8

    }

    fn minute(&self) -> u8 {
        // Bits 10-5 represent the minutes
        ((self.time.0 >> 5) & 0x3F) as u8
    }

    fn second(&self) -> u8 {
        // Bits 4-0 represent the seconds divided by 2
        (2 * (self.time.0 & 0x1F)) as u8 
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02} on {:02}/{:02}/{:04}",
            self.hour(),
            self.minute(),
            self.second(),
            self.month(),
            self.day(),
            self.year()
        )
    }
}

impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    fn read_only(&self) -> bool {
        self.attributes.0.bitand(0x01) == 0x01
    }

    fn hidden(&self) -> bool {
        self.attributes.0.bitand(0x02) == 0x02
    }

    fn created(&self) -> Self::Timestamp {
        Timestamp {
            date: self.created_date,
            time: self.created_time,
        }
    }

    fn accessed(&self) -> Self::Timestamp {
        Timestamp {
            date: self.accessed_date,
            time: Time(0), // no info, conservatively default to 00:00:00
        }
    }

    fn modified(&self) -> Self::Timestamp {
        Timestamp {
            date: self.modified_date,
            time: self.modified_time,
        }
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Created @ {}\nLast Modified @ {}\nLast Accessed on {:02}/{:02}/{:04}\nHidden: {}\nRead-Only: {}",
            self.created(),
            self.modified(),
            self.accessed().day(),
            self.accessed().month(),
            self.accessed().year(),
            self.hidden(),
            self.read_only()
        )
    }
}