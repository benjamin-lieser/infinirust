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
    /// A block has been updated
    Block([i32; 3], u8),
    /// Exit the game
    Exit,
}

enum Package {
    Chunk([i32; 3], Vec<u8>),
}

pub fn chunk_loader(
    tcp: TcpStream,
    world: Arc<World>,
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

        let read_join_handle = tokio::spawn(read_packages(reader, loader_tx));
        let write_join_handle = tokio::spawn(write_packages(writer, writer_rx));

        let world_join_handler = tokio::spawn(manage_world(world, atlas, loader_rx, writer_tx, updates));
        // When manage_world returns, the client has exited
        world_join_handler.await.unwrap();
        read_join_handle.abort();
        write_join_handle.abort();
    });

    rt.shutdown_background();
}

async fn manage_world(
    world: Arc<World>,
    atlas: Arc<TextureAtlas>,
    mut in_packages: tokio::sync::mpsc::Receiver<Package>,
    out_packages: tokio::sync::mpsc::Sender<Box<[u8]>>,
    mut client: tokio::sync::mpsc::Receiver<Update>,

) {
    loop {
        tokio::select! {
            package = in_packages.recv() => {
                match package {
                    // Chunkdata recieved
                    Some(Package::Chunk(pos, data)) => {
                        // Both locks in this section are sync, but we do not await here
                        let mut chunk = {
                            let mut unused_chunks_rx = world.unused_chunks.lock().unwrap();
                            unused_chunks_rx.pop().expect("No available chunks")
                        };
                        chunk.load(data, pos);
                        chunk.write_vbo(&atlas);
                        let mut chunks = world.chunks.lock().unwrap();
                        chunks.push(chunk);
                    }
                    None => {panic!("package reader crashed")}
                }
            }
            update = client.recv() => {
                match update {
                    Some(Update::Pos(camera)) => {

                    }
                    Some(Update::Block(pos, block)) => {

                    }
                    Some(Update::Exit) => {
                        return;
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
