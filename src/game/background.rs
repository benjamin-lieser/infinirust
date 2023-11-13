use std::{net::TcpStream, sync::{Mutex, Arc}};

use crate::mygl::TextureAtlas;

use super::{World, FreeCamera};

pub fn chunk_loader(tcp: TcpStream, world: Mutex<World>, pos_updates: tokio::sync::mpsc::Receiver<FreeCamera>, atlas: Arc<TextureAtlas>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        tcp.set_nonblocking(true).unwrap();
        let tcp = tokio::net::TcpStream::from_std(tcp).unwrap();
        let (reader, writer) = tcp.into_split();

    });
}