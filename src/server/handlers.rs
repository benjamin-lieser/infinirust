use crate::misc::AsBytes;

#[repr(C)]
#[derive(Debug, Default)]
pub struct PackageBlockUpdate {
    pub pos: [i32; 3],
    pub block: u8,
    pub reserved: [u8; 3],
}

unsafe impl AsBytes for PackageBlockUpdate {}
