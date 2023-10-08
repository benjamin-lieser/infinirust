use std::sync::{Arc, Mutex};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener, TcpStream,
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
        let world = ServerWorld::new(42);

        let (command_tx, command_rx) = tokio::sync::mpsc::channel(10);

        let (write_package_tx, _) = tokio::sync::broadcast::channel(10);


        let write_package_tx2 = write_package_tx.clone();        
        std::thread::spawn(|| infinirust::server::manage_chunk_data(command_rx, write_package_tx2, world));

        // accept connections and process them in a new thread
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let (read, write) = stream.into_split();

            tokio::task::spawn(write_packages(write, write_package_tx.subscribe()));
            tokio::task::spawn(read_packages(read, command_tx.clone()));
        }
    });

    Ok(())
}

async fn write_packages(
    mut stream: OwnedWriteHalf,
    mut input: tokio::sync::broadcast::Receiver<Arc<[u8]>>,
) {
    loop {
        let package = input.recv().await.unwrap();
        //TODO: logic to discard packages which are not relevant to the loaded chunks of the client
        stream.write_all(&package).await.unwrap();
    }
}

async fn read_packages(mut stream: OwnedReadHalf, output: tokio::sync::mpsc::Sender<Command>) {
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
                let command = Command::ChunkData(pos);
                output.send(command).await.unwrap();
            }
            // Request chunk data
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
