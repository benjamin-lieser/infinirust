use std::sync::Arc;

use tokio::{io::AsyncReadExt, net::tcp::OwnedReadHalf};

use crate::{misc::{cast_bytes, cast_bytes_mut, AsBytes}, server::{Command, ServerCommand, UID}};

pub trait Package: Default + AsBytes {
    fn id() -> u16;
    async fn new(stream: &mut OwnedReadHalf) -> Self {
        let mut package = Self::default();
        stream
            .read_exact(cast_bytes_mut(&mut package))
            .await
            .unwrap();
        package
    }

    async fn handle(&self, command: &ServerCommand, uid : UID);

    async fn read_and_handle(
        stream: &mut OwnedReadHalf,
        command: &ServerCommand,
        uid : UID
    ) {
        let package = Self::new(stream).await;
        package.handle(command, uid).await;
    }

    fn to_arc(&self) -> Arc<[u8]> {
        let mut bytes = vec![0u8; std::mem::size_of::<Self>() + 2];
        bytes[0..2].copy_from_slice(&Self::id().to_le_bytes());
        bytes[2..].copy_from_slice(cast_bytes(self));
        bytes.into()
    }
    fn to_box(&self) -> Box<[u8]> {
        let mut bytes = vec![0u8; std::mem::size_of::<Self>() + 2];
        bytes[0..2].copy_from_slice(&Self::id().to_le_bytes());
        bytes[2..].copy_from_slice(cast_bytes(self));
        bytes.into()
    }
}

impl Package for ClientPackagePlayerPosition {
    fn id() -> u16 {
        0x000C
    }
    async fn handle(&self, command: &ServerCommand, uid : UID) {
        command
            .send((uid, Command::PlayerPosition(
                self.pos, self.pitch, self.yaw,
            )))
            .await
            .unwrap();
    }
}

impl Package for ServerPackagePlayerPosition {
    fn id() -> u16 {
        0x000C
    }
    async fn handle(&self, _command: &ServerCommand, _uid : UID) {
        panic!("ServerPackagePlayerPosition should not be received by the server");
    }

}

#[repr(C)]
#[derive(Debug, Default)]
pub struct PackageBlockUpdate {
    pub pos: [i32; 3],
    pub block: u8,
    pub reserved: [u8; 3],
}
unsafe impl AsBytes for PackageBlockUpdate {}
const _: () = assert!(
    std::mem::size_of::<PackageBlockUpdate>() % std::mem::align_of::<PackageBlockUpdate>() == 0
);

#[repr(C)]
#[derive(Debug, Default)]
pub struct ClientPackagePlayerPosition {
    pub pos: [f64; 3],
    pub pitch: f32,
    pub yaw: f32,
}
unsafe impl AsBytes for ClientPackagePlayerPosition {}
const _: () = assert!(
    std::mem::size_of::<ClientPackagePlayerPosition>()
        % std::mem::align_of::<ClientPackagePlayerPosition>()
        == 0
);

#[repr(C)]
#[derive(Debug, Default)]
pub struct ServerPackagePlayerPosition {
    pub uid: u64,
    pub pos: [f64; 3],
    pub pitch: f32,
    pub yaw: f32,
}
unsafe impl AsBytes for ServerPackagePlayerPosition {}
const _: () = assert!(
    std::mem::size_of::<ServerPackagePlayerPosition>()
        % std::mem::align_of::<ServerPackagePlayerPosition>()
        == 0
);
