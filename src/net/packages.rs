use std::sync::Arc;

use tokio::{io::AsyncReadExt, net::tcp::OwnedReadHalf};
use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::server::{Command, ServerCommand, UID};

pub trait Package: Default + IntoBytes + FromBytes + Immutable {
    fn id() -> u16;
    async fn new(stream: &mut OwnedReadHalf) -> Self {
        let mut package = Self::default();
        stream.read_exact(package.as_mut_bytes()).await.unwrap();
        package
    }

    async fn handle(&self, command: &ServerCommand, uid: UID);

    async fn read_and_handle(stream: &mut OwnedReadHalf, command: &ServerCommand, uid: UID) {
        let package = Self::new(stream).await;
        package.handle(command, uid).await;
    }

    fn to_arc(&self) -> Arc<[u8]> {
        let mut bytes = vec![0u8; std::mem::size_of::<Self>() + 2];
        bytes[0..2].copy_from_slice(&Self::id().to_le_bytes());
        bytes[2..].copy_from_slice(self.as_bytes());
        bytes.into()
    }
    fn to_box(&self) -> Box<[u8]> {
        let mut bytes = vec![0u8; std::mem::size_of::<Self>() + 2];
        bytes[0..2].copy_from_slice(&Self::id().to_le_bytes());
        bytes[2..].copy_from_slice(self.as_bytes());
        bytes.into()
    }
}

impl Package for ClientPackagePlayerPosition {
    fn id() -> u16 {
        0x000C
    }
    async fn handle(&self, command: &ServerCommand, uid: UID) {
        command
            .send((uid, Command::PlayerPosition(self.pos, self.pitch, self.yaw)))
            .await
            .unwrap();
    }
}

impl Package for ServerPackagePlayerPosition {
    fn id() -> u16 {
        0x000C
    }
    async fn handle(&self, _command: &ServerCommand, _uid: UID) {
        panic!("ServerPackagePlayerPosition should not be received by the server");
    }
}

impl Package for PackageBlockUpdate {
    fn id() -> u16 {
        0x000B
    }
    async fn handle(&self, _command: &ServerCommand, uid: UID) {
        todo!("Handle block update for uid: {uid}");
    }
}

impl Package for ServerPackageLogout {
    fn id() -> u16 {
        0x0004
    }
    async fn handle(&self, _command: &ServerCommand, _uid: UID) {
        panic!("ServerPackageLogout should not be received by the server");
    }
}

#[repr(C)]
#[derive(Debug, Default, IntoBytes, FromBytes, Immutable)]
pub struct PackageBlockUpdate {
    pub pos: [i32; 3],
    pub block: u8,
    pub reserved: [u8; 3],
}

#[repr(C)]
#[derive(Debug, Default, IntoBytes, FromBytes, Immutable)]
pub struct ClientPackagePlayerPosition {
    pub pos: [f64; 3],
    pub pitch: f32,
    pub yaw: f32,
}

#[repr(C)]
#[derive(Debug, Default, IntoBytes, FromBytes, Immutable)]
pub struct ServerPackagePlayerPosition {
    pub uid: u64,
    pub pos: [f64; 3],
    pub pitch: f32,
    pub yaw: f32,
}

#[repr(C)]
#[derive(Debug, Default, IntoBytes, FromBytes, Immutable)]
pub struct ServerPackageLogout {
    pub uid: u64,
}

pub struct ServerPlayerLogin {
    pub uid: u64,
    pub name: String,
}

impl ServerPlayerLogin {
    pub fn to_arc(&self) -> Arc<[u8]> {
        assert!(self.name.len() <= u16::MAX as usize);
        let mut bytes = vec![0u8; 2 + 8 + 2 + self.name.len()];
        bytes[0..2].copy_from_slice(&3u16.to_le_bytes());
        bytes[2..4].copy_from_slice(&(self.name.len() as u16).to_le_bytes());
        bytes[4..4 + self.name.len()].copy_from_slice(self.name.as_bytes());
        bytes[4 + self.name.len()..].copy_from_slice(self.uid.as_bytes());
        bytes.into()
    }
    pub async fn new(stream: &mut OwnedReadHalf) -> Self {
        let mut name_len = [0u8; 2];
        stream.read_exact(&mut name_len).await.unwrap();
        let name_len = u16::from_le_bytes(name_len) as usize;
        let mut name = vec![0u8; name_len];
        stream.read_exact(&mut name).await.unwrap();
        let name = String::from_utf8(name).unwrap();
        let mut uid = [0u8; 8];
        stream.read_exact(&mut uid).await.unwrap();
        let uid = u64::from_le_bytes(uid);
        Self { uid, name }
    }
}
