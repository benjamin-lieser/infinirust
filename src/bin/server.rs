use anyhow::anyhow;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener,
};

use infinirust::misc::cast_bytes_mut;
use infinirust::server::handlers::PackageBlockUpdate;
use infinirust::server::{BlockUpdateMode, Client, Command, UID};

fn main() -> std::io::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:8042").await.unwrap();

        let (command_tx, command_rx) = tokio::sync::mpsc::channel(100);

        std::thread::spawn(|| infinirust::server::start_world(command_rx, "world".into()));

        // accept connections and process them in a new task
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let (read, write) = stream.into_split();

            let (write_tx, write_rx) = tokio::sync::mpsc::channel(100);

            tokio::task::spawn(write_packages(write, write_rx));
            tokio::task::spawn(read_start_packages(read, command_tx.clone(), write_tx));
        }
    });

    std::io::Result::Ok(())
}

async fn write_packages(
    mut stream: OwnedWriteHalf,
    mut input: tokio::sync::mpsc::Receiver<Arc<[u8]>>,
) {
    loop {
        //This unwraps panics iff the user is logged out
        let package = input.recv().await.unwrap();

        stream.write_all(&package).await.unwrap();
    }
}

/// Read the packages when the server is in `play` state
/// It will never return Ok. todo never type if it gets stable
/// If it returns Err the user has to be logged out
async fn read_play_packages(
    mut stream: OwnedReadHalf,
    server: tokio::sync::mpsc::Sender<Command>,
    uid: UID,
) -> Result<(), anyhow::Error> {
    loop {
        let mut package_type = [0u8; 2];
        stream.read_exact(&mut package_type).await?;

        match u16::from_le_bytes(package_type) {
            // Request chunk data
            0x000A => {
                let mut pos = [0i32; 3];
                stream
                    .read_exact(infinirust::misc::as_bytes_mut(&mut pos))
                    .await?;
                let command = Command::ChunkData(pos, uid);
                server
                    .send(command)
                    .await
                    .expect("This should never happen. The internal server is not responding");
            }
            // Send block update
            0x000B => {
                // Send block update
                let mut package = PackageBlockUpdate::default();
                stream.read_exact(cast_bytes_mut(&mut package)).await?;

                server
                    .send(Command::BlockUpdate(package.pos, package.block))
                    .await
                    .expect("This should never happen. The internal server is not responding");
            }
            _ => {
                return Err(anyhow!("Invalid package type"));
            }
        }
    }
}

/// Read the packages when the server is in `start` state
async fn read_start_packages(
    mut stream: OwnedReadHalf,
    server: tokio::sync::mpsc::Sender<Command>,
    client: Client,
) {
    let uid = loop {
        let mut package_type = [0u8; 2];
        stream.read_exact(&mut package_type).await.unwrap();
        match u16::from_le_bytes(package_type) {
            // Login
            0x0001 => {
                let (tx, rx) = tokio::sync::oneshot::channel();

                if let Some(username) = read_alpha_numeric_string(&mut stream).await {
                    let command = Command::Login(username, client.clone(), tx);
                    server.send(command).await.unwrap();

                    if let Some(uid) = rx.await.unwrap() {
                        break uid; //Move on to play state
                    }
                }
                //Login unsuccessful

                //Send Login failed package with empty message. todo
                client
                    .send((b"\x01\x00\x00\x00" as &[u8]).into())
                    .await
                    .unwrap();
                //Do not revieve anymore packages
                return;
            }
            _ => {
                panic!("Recieved invalid package for state `start`");
            }
        }
    };
    //Go to play state
    read_play_packages(stream, server.clone(), uid)
        .await
        .expect_err("Somehow the read_play_packages function returned with Ok");
    //Log the player out
    server.send(Command::Logout(uid)).await.unwrap();
}

async fn read_alpha_numeric_string(stream: &mut OwnedReadHalf) -> Option<String> {
    let mut length = [0u8; 2];
    stream.read_exact(&mut length).await.unwrap();

    let length = u16::from_le_bytes(length);

    let mut string = vec![0u8; length as usize];

    stream.read_exact(&mut string).await.unwrap();

    let string = String::from_utf8(string).ok()?;

    if string.chars().all(char::is_alphanumeric) {
        return Some(string);
    } else {
        return None;
    }
}
