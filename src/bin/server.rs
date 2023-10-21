use std::sync::Arc;
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener
};

use infinirust::server::{world::ServerWorld, Command, UUID, Client};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn main() -> std::io::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:8042").await.unwrap();
        //common shared state

        let (command_tx, command_rx) = tokio::sync::mpsc::channel(10);

        std::thread::spawn(|| infinirust::server::start_world(command_rx, "world".into()));

        // accept connections and process them in a new thread
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let (read, write) = stream.into_split();

            let (write_tx, write_rx) = tokio::sync::mpsc::channel(10);

            tokio::task::spawn(write_packages(write, write_rx));
            tokio::task::spawn(read_start_packages(read, command_tx.clone(), write_tx));
        }
    });

    Ok(())
}

async fn write_packages(
    mut stream: OwnedWriteHalf,
    mut input: tokio::sync::mpsc::Receiver<Arc<[u8]>>,
) {
    loop {
        let package = input.recv().await.unwrap();
        //TODO: logic to discard packages which are not relevant to the loaded chunks of the client
        stream.write_all(&package).await.unwrap();
    }
}

/// Read the packages when the server is in `play` state
async fn read_play_packages(
    mut stream: OwnedReadHalf,
    output: tokio::sync::mpsc::Sender<Command>,
    uuid: UUID,
) {
    loop {
        let mut package_type = [0u8; 2];
        stream.read_exact(&mut package_type).await.unwrap();
        eprintln!("{:?}", package_type);
        match u16::from_le_bytes(package_type) {
            // Request chunk data
            0x0A => {
                let mut pos = [0i32; 3];
                stream
                    .read_exact(infinirust::misc::as_bytes_mut(&mut pos))
                    .await
                    .unwrap();
                let command = Command::ChunkData(pos, uuid);
                output.send(command).await.unwrap();
            }
            // Send block update
            0x0B => {
                // Send block update
                let mut pos = [0i32; 3];
                stream
                    .read_exact(infinirust::misc::as_bytes_mut(&mut pos))
                    .await
                    .unwrap();
            }
            _ => {
                panic!("Invalid package type")
            }
        }
    }
}

/// Read the packages when the server is in `start` state
async fn read_start_packages(
    mut stream: OwnedReadHalf,
    output: tokio::sync::mpsc::Sender<Command>,
    client: Client,
) {
    loop {
        let mut package_type = [0u8; 2];
        stream.read_exact(&mut package_type).await.unwrap();
        match u16::from_le_bytes(package_type) {
            // Login
            0x0001 => {
                let mut length = [0u8; 2];
                stream.read_exact(&mut length).await.unwrap();

                let length = u16::from_le_bytes(length);

                let mut string = vec![0u8;length as usize];

                stream.read_exact(&mut string).await.unwrap();

                let name = String::from_utf8(string).unwrap();

                let (tx, rx) = tokio::sync::oneshot::channel();

                let command = Command::Login(name, client.clone(), tx);
                output.send(command).await.unwrap();

                let uuid = rx.await.unwrap();

                //todo

            }
            _ => {}
        }
    }
}