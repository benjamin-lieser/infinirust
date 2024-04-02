use std::sync::Arc;

use tokio::{io::AsyncReadExt, net::tcp::OwnedReadHalf};

use crate::misc::{cast_bytes_mut, cast_bytes, AsBytes};

pub trait Package {
    fn id() -> u16;
    async fn new(stream: &mut OwnedReadHalf) -> Self;
    async fn handle(&self, command: &mut tokio::sync::mpsc::Sender<crate::server::Command>);
    async fn read_and_handle(
        stream: &mut OwnedReadHalf,
        command: &mut tokio::sync::mpsc::Sender<crate::server::Command>,
    ) where
        Self: Sized,
    {
        let package = Self::new(stream).await;
        package.handle(command).await;
    }
    fn to_bytes(&self) -> Arc<[u8]> 
    where
        Self: AsBytes{
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
    async fn new(stream: &mut OwnedReadHalf) -> Self {
        let mut package = ClientPackagePlayerPosition::default();
        stream
            .read_exact(cast_bytes_mut(&mut package))
            .await
            .unwrap();
        package
    }
    async fn handle(&self, command: &mut tokio::sync::mpsc::Sender<crate::server::Command>) {
        command
            .send(crate::server::Command::PlayerPosition(
                self.pos, self.pitch, self.yaw,
            ))
            .await
            .unwrap();
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
