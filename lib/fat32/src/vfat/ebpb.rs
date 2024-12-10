use shim::const_assert_size;

use crate::traits::BlockDevice;
use crate::vfat::Error;

#[repr(C, packed)]
#[derive(Debug)]
pub struct BiosParameterBlock {
    jmp_instruction: [u8; 3],
    oem_identifier: u64,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub num_reserved_sectors: u16,
    pub num_fats: u8,
    max_dir_entries: u16,
    small_total_logical_sectors: u16,
    fat_id: u8,
    old_spf: u16, // sectors per fat previous fat32
    sectors_per_track: u16,
    num_heads: u16,
    num_hidden_sectors : u32, // LBA of the beginning of the partition
    num_logical_sectors : u32,
    pub sectors_per_fat : u32,
    flags : u16,
    version : u16,
    pub root_dir_cluster : u32,
    fsinfo_sector : u16,
    backup_boot_sector : u16,
    _reserved : [u8; 12],
    drive_num : u8,
    _reserved_win : u8,
    signature : u8,
    volume_id : u32,
    volume_label : [u8; 11],
    sys_identifier: u64,
    boot_code : [u8; 420],
    boot_signature : [u8; 2]
    // pub start: u64,
    // pub num_sectors: u64,
    // pub sector_size: u64,
}

impl Default for BiosParameterBlock {
    fn default() -> BiosParameterBlock {
        BiosParameterBlock {
            jmp_instruction: [0; 3],
            oem_identifier: 0,
            bytes_per_sector: 0,
            sectors_per_cluster: 0,
            num_reserved_sectors: 0,
            num_fats: 0,
            max_dir_entries: 0,
            small_total_logical_sectors: 0,
            fat_id: 0,
            old_spf: 0,
            sectors_per_track: 0,
            num_heads: 0,
            num_hidden_sectors: 0,
            num_logical_sectors: 0,
            sectors_per_fat: 0,
            flags: 0,
            version: 0,
            root_dir_cluster: 0,
            fsinfo_sector: 0,
            backup_boot_sector: 0,
            _reserved: [0; 12],
            drive_num: 0,
            _reserved_win: 0,
            signature: 0,
            volume_id: 0,
            volume_label: [0; 11],
            sys_identifier: 0,
            boot_code: [0; 420],
            boot_signature: [0; 2],
        }
    }
}

const_assert_size!(BiosParameterBlock, 512);
const VALID_SIGNATURE: [u8; 2] = [0x55, 0xAA];

use alloc::slice;

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(mut device: T, sector: u64) -> Result<BiosParameterBlock, Error> {

        let mut ebpb: BiosParameterBlock = BiosParameterBlock::default();
        let ebpb_slice: &mut [u8] = unsafe {
            slice::from_raw_parts_mut(
                &mut ebpb as *mut BiosParameterBlock as *mut u8,
                size_of::<BiosParameterBlock>(),
            )
        };
        device.read_sector(sector, ebpb_slice).map_err(|err| Error::Io(err))?;
        if ebpb.boot_signature != VALID_SIGNATURE {
            return Err(Error::BadSignature);
        }
        

        Ok(ebpb)
    }
}