use std::{collections::HashMap, net::TcpStream, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};
use zerocopy::IntoBytes;

use crate::{
    game::{
        chunk::block_position_to_chunk_index, world::VIEW_DISTANCE, Camera, CHUNK_SIZE, Y_RANGE,
    },
    misc::first_none,
    mygl::TextureAtlas,
    net::{
        ClientPackagePlayerPosition, Package as NetworkPackage, ServerPackagePlayerPosition,
        ServerPlayerLogin,
    },
    server::UID,
};

use super::{FreeCamera, World};

/// Updates which are send from the main loop to the background thread
#[derive(Debug)]
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
    PlayerPositionUpdate(ServerPackagePlayerPosition),
    PlayerLogin(ServerPlayerLogin),
}

pub fn background_thread(
    tcp: TcpStream,
    world: Arc<World>,
    updates: tokio::sync::mpsc::Receiver<Update>,
    atlas: Arc<TextureAtlas>,
    uid: UID,
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

        let (loader_tx, loader_rx) = tokio::sync::mpsc::channel(10000);
        let (writer_tx, writer_rx) = tokio::sync::mpsc::channel(10000);

        let read_join_handle = tokio::spawn(read_packages(reader, loader_tx));
        let write_join_handle = tokio::spawn(write_packages(writer, writer_rx));

        let world_join_handler = tokio::spawn(manage_world(
            world, atlas, loader_rx, writer_tx, updates, uid,
        ));
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
    uid: UID,
) {
    let mut current_world_center = [i32::MIN; 2]; // x, z
    let mut active_chunk_ids = HashMap::<[i32; 3], usize>::new();

    loop {
        tokio::select! {
            package = in_packages.recv() => {
                match package {
                    // Chunkdata recieved
                    Some(Package::Chunk(pos, data)) => {
                        // Both locks in this section are sync, but we do not await here
                        let mut unused_chunks_rx = world.unused_chunks.lock().unwrap();
                        let mut chunk = {
                            unused_chunks_rx.pop().expect("No available chunks")
                        };
                        chunk.load(data, pos);
                        chunk.write_vbo(&atlas);
                        {
                            // This lock is time critical for the renderer thread, so be quick about it
                            let mut chunks = world.chunks.lock().unwrap();
                            let slot = first_none(&chunks).expect("No available slot, this should be impossible");

                            if let Some(slot) = active_chunk_ids.insert(pos, slot) {
                                unused_chunks_rx.push(chunks[slot].take().expect("There should be a chunk in this slot"));

                            }
                            chunks[slot] = Some(chunk);
                        }
                    }
                    Some(Package::PlayerPositionUpdate(package)) => {
                        // Update player position
                        world.players.lock().unwrap().update(&package);
                        if package.uid as UID == uid {
                            // Force update
                            current_world_center = [i32::MIN; 2];
                        }
                    }
                    Some(Package::PlayerLogin(package)) => {
                        // Add player to world
                        world.players.lock().unwrap().add_player(package.name, package.uid as UID, FreeCamera::new([0.0,0.0,0.0]));
                    }
                    None => {panic!("package reader crashed")}
                }
            }
            update = client.recv() => {
                match update {
                    Some(Update::Pos(camera)) => {
                        let position_package = ClientPackagePlayerPosition {
                            pos: camera.position(),
                            pitch: camera.pitch(),
                            yaw: camera.yaw(),
                        };
                        out_packages.send(position_package.to_box()).await.unwrap();
                        let camera_pos = camera.position();
                        let camera_center = [
                            camera_pos[0] as i32 / CHUNK_SIZE as i32,
                            camera_pos[2] as i32 / CHUNK_SIZE as i32,
                        ];
                        if camera_center != current_world_center {
                            // Remove chunks that are too far away
                            {
                                let mut unused_chunks = world.unused_chunks.lock().unwrap();
                                let mut chunks = world.chunks.lock().unwrap();
                                active_chunk_ids.retain(|pos, slot| {
                                    if (pos[0] - camera_center[0]).abs() > VIEW_DISTANCE || (pos[2] - camera_center[1]).abs() > VIEW_DISTANCE {
                                        unused_chunks.push(chunks[*slot].take().expect("There should be a chunk in this slot"));
                                        false
                                    } else {
                                        true
                                    }
                                });
                            }
                            // Load new chunks
                            for x in -VIEW_DISTANCE..=VIEW_DISTANCE {
                                for z in -VIEW_DISTANCE..=VIEW_DISTANCE {
                                    let pos = [camera_center[0] + x, 0, camera_center[1] + z];
                                    if !active_chunk_ids.contains_key(&pos) {
                                        for y in -Y_RANGE..Y_RANGE {
                                            let pos = [pos[0], y, pos[2]];
                                            out_packages.send(request_chunk_package(pos)).await.unwrap();
                                        }
                                    }
                                }
                            }
                            current_world_center = camera_center;
                        }
                    }
                    Some(Update::Block(pos, block)) => {
                        let (chunk_index, block_index) = block_position_to_chunk_index(pos);
                        if let Some(slot) = active_chunk_ids.get(&chunk_index) {
                            let mut chunks = world.chunks.lock().unwrap();
                            if let Some(chunk) = &mut chunks[*slot] {
                                chunk.update_block(block_index, block, &atlas);
                            }
                        }
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

fn request_chunk_package(pos: [i32; 3]) -> Box<[u8]> {
    let mut package = vec![0u8; 14];
    package[0..2].copy_from_slice(0x000Au16.as_bytes());
    package[2..].copy_from_slice(pos.as_bytes());
    package.into_boxed_slice()
}

async fn read_packages(
    mut reader: OwnedReadHalf,
    chunk_loader: tokio::sync::mpsc::Sender<Package>,
) {
    let mut package_type = 0u16;
    loop {
        reader
            .read_exact(package_type.as_mut_bytes())
            .await
            .unwrap();

        match package_type {
            0x000A => {
                //Chunk Data
                let mut pos = [0i32; 3];
                reader.read_exact(pos.as_mut_bytes()).await.unwrap();
                let mut data = vec![0u8; 4096];
                reader.read_exact(&mut data).await.unwrap();
                chunk_loader.send(Package::Chunk(pos, data)).await.unwrap();
            }
            0x000B => {
                //Block Update
                todo!();
            }
            0x000C => {
                let player_pos = ServerPackagePlayerPosition::new(&mut reader).await;
                chunk_loader
                    .send(Package::PlayerPositionUpdate(player_pos))
                    .await
                    .unwrap();
            }
            0x0003 => {
                // Other player logs in
                let login_package = ServerPlayerLogin::new(&mut reader).await;
                chunk_loader
                    .send(Package::PlayerLogin(login_package))
                    .await
                    .unwrap();
            }
            _ => {
                panic!("Client: Invalid Package type {package_type}")
            }
        }
    }
}
