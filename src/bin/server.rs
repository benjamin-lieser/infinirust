use std::sync::Arc;
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener
};

use infinirust::server::{world::ServerWorld, Command};
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
            tokio::task::spawn(read_packages(read, command_tx.clone(), write_tx));
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

async fn read_packages(
    mut stream: OwnedReadHalf,
    output: tokio::sync::mpsc::Sender<Command>,
    write_channel: tokio::sync::mpsc::Sender<Arc<[u8]>>,
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
                let command = Command::ChunkData(pos, write_channel.clone());
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
