use crate::misc::AsBytes;

#[repr(C)]
#[derive(Debug, Default)]
pub struct PackageBlockUpdate {
    pub pos : [i32;3],
    pub placed : u8,
    pub block : u8
}

unsafe impl AsBytes for PackageBlockUpdate {}
//TODO padding in PackageBlockUpdate