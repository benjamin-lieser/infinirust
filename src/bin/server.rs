use anyhow::anyhow;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener,
};

use infinirust::misc::{cast_bytes_mut, cast_bytes};
use infinirust::server::handlers::PackageBlockUpdate;
use infinirust::server::{Client, Command, UID};

fn main() -> std::io::Result<()> {

    let args: Vec<String> = std::env::args().collect();

    let listen_on = args[1].clone();
    let world_directory = args[2].clone();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let (bind, listener) = if listen_on == "internal" { //Bind to loopback and let the OS assign a port
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            (addr.to_string(), listener)
        } else {
            let bind = listen_on.clone();
            (bind, TcpListener::bind(listen_on).await.unwrap())
        };

        let (command_tx, command_rx) = tokio::sync::mpsc::channel(100);

        std::thread::spawn(|| infinirust::server::start_world(command_rx, world_directory.into()));
        
        let stdin_command_tx = command_tx.clone();
        std::thread::spawn(|| infinirust::server::stdin::handle_stdin(stdin_command_tx, bind));

        let server_ctrlc = command_tx.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            //Send shutdown command to the server. If the server is already gone it exits the process
            eprintln!("Server recieved ctrl+C");
            server_ctrlc.send(Command::Shutdown).await.unwrap_or_else(|_| std::process::exit(1));
        });

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
    //TODO Consider accepting a package enum instead of the written out packages
    loop {
        //This unwraps panics iff the user is logged out
        if let Some(package) = input.recv().await {
            stream.write_all(&package).await.unwrap();
        } else {
            eprintln!("Writer returns");
            return;
        }
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
                    .read_exact(infinirust::misc::cast_bytes_mut(&mut pos))
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
                        //TODO send login success package
                        let mut package = vec![0x02u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                        package[2..].copy_from_slice(cast_bytes(&uid));
                        client.send(package.into()).await.unwrap();

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

    let e = read_play_packages(stream, server.clone(), uid)
        .await
        .expect_err("Somehow the read_play_packages function returned with Ok");
    //Log the player out
    eprintln!("Player got logged out because of error: {}",e);
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
