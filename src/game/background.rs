use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

use crate::{game::Chunk, misc::cast_bytes_mut, mygl::TextureAtlas};

use super::{FreeCamera, World};

pub enum Update {
    /// The camera position has changed
    Pos(FreeCamera),
}

enum Package {
    Chunk([i32; 3], Vec<u8>),
}

pub fn chunk_loader(
    tcp: TcpStream,
    world: Arc<Mutex<World>>,
    updates: tokio::sync::mpsc::Receiver<Update>,
    atlas: Arc<TextureAtlas>,
) {
    // Start a single thread tokio runtime in this thread
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        tcp.set_nonblocking(true).unwrap();
        let tcp = tokio::net::TcpStream::from_std(tcp).unwrap();
        let (reader, writer) = tcp.into_split();

        let (loader_tx, loader_rx) = tokio::sync::mpsc::channel(100);
        let (writer_tx, writer_rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(read_packages(reader, loader_tx));
        tokio::spawn(write_packages(writer, writer_rx));
    });
}

async fn manage_world(
    world: Arc<Mutex<World>>,
    atlas: Arc<TextureAtlas>,
    mut packages: tokio::sync::mpsc::Receiver<Package>,
    mut client: tokio::sync::mpsc::Receiver<Update>,
) {
    loop {
        tokio::select! {
            package = packages.recv() => {
                match package {
                    Some(Package::Chunk(pos, data)) => {
                        
                    }
                    None => {panic!("package reader crashed")}
                }
            }
            update = client.recv() => {
                match update {
                    Some(Update::Pos(camera)) => {

                    }
                    None => {panic!("client crashed")}
                }
            }
        }
    }
}

async fn write_packages(
    mut stream: OwnedWriteHalf,
    mut input: tokio::sync::mpsc::Receiver<Box<[u8]>>,
) {
    loop {
        if let Some(package) = input.recv().await {
            stream.write_all(&package).await.unwrap();
        } else {
            eprintln!("Client: Writer returns");
            return;
        }
    }
}

async fn read_packages(
    mut reader: OwnedReadHalf,
    chunk_loader: tokio::sync::mpsc::Sender<Package>,
) {
    let mut package_type = 0u16;
    loop {
        reader
            .read_exact(cast_bytes_mut(&mut package_type))
            .await
            .unwrap();

        match package_type {
            0x000A => {
                //Chunk Data
                let mut pos = [0i32; 3];
                reader.read_exact(cast_bytes_mut(&mut pos)).await.unwrap();
                let mut data = vec![0u8; 4096];
                reader.read_exact(&mut data).await.unwrap();
                chunk_loader.send(Package::Chunk(pos, data)).await.unwrap();
            }
            0x0000B => {
                //Block Update
                todo!();
            }
            _ => {
                panic!("Client: Invalid Package type")
            }
        }
    }
}
