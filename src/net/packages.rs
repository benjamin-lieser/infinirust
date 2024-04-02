use crate::misc::AsBytes;

#[repr(C)]
#[derive(Debug, Default)]
pub struct PackageBlockUpdate {
    pub pos: [i32; 3],
    pub block: u8,
    pub reserved: [u8; 3],
}
unsafe impl AsBytes for PackageBlockUpdate {}
const _ : () = assert!(std::mem::size_of::<PackageBlockUpdate>() % std::mem::align_of::<PackageBlockUpdate>() == 0);

#[repr(C)]
#[derive(Debug, Default)]
pub struct ClientPackagePlayerPosition {
    pub pos: [f64; 3],
    pub pitch : f32,
    pub yaw : f32,
}
unsafe impl AsBytes for ClientPackagePlayerPosition {}
const _ : () = assert!(std::mem::size_of::<ClientPackagePlayerPosition>() % std::mem::align_of::<ClientPackagePlayerPosition>() == 0);

#[repr(C)]
#[derive(Debug, Default)]
pub struct ServerPackagePlayerPosition {
    pub uid: u64,
    pub pos: [f64; 3],
    pub pitch : f32,
    pub yaw : f32,
}
unsafe impl AsBytes for ServerPackagePlayerPosition {}
const _ : () = assert!(std::mem::size_of::<ServerPackagePlayerPosition>() % std::mem::align_of::<ServerPackagePlayerPosition>() == 0);