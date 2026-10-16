#[repr(C)]
pub struct BootInfo {
    pub(crate) magic: u32,
    pub(crate) version: u32,
    pub(crate) size: u32,
    pub(crate) e820_count: u32,
    pub(crate) e820_entry_size: u32,
    pub(crate) e820_address: u32,
    pub(crate) boot_drive: u32,
    pub(crate) reserved: u32,
}

const BOOT_INFO_MAGIC: u32 = 0x49424b4f;

pub(crate) fn validate(boot_info: *const BootInfo) -> Option<&'static BootInfo> {

    if boot_info as usize != 0x0600 {
        return None;
    }

    let info: &'static BootInfo = unsafe { &*boot_info };

    if info.magic == BOOT_INFO_MAGIC
        && info.version == 1
        && info.size == 32
        && (1..=32).contains(&info.e820_count)
        && info.e820_entry_size == 24
        && info.e820_address == 0x5000
        && info.boot_drive <= 0xff
        && info.reserved == 0
    {
        Some(info)
    } else {
        None
    }
}