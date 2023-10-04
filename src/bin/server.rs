use std::sync::{Arc, Mutex};
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

use infinirust::game::ServerWorld;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8042")?;

    //common shared state
    let world = Arc::new(Mutex::new(ServerWorld::new(42)));

    // accept connections and process them in a new thread
    for stream in listener.incoming() {
        let stream = stream?;
        let world = world.clone();
        std::thread::spawn(move || {
            handle_connection(stream, world.clone());
        });
    }
    Ok(())
}

fn handle_connection(stream: TcpStream, world: Arc<Mutex<ServerWorld>>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let local = tokio::task::LocalSet::new();
        stream.set_nonblocking(true).unwrap();
        let mut stream = tokio::net::TcpStream::from_std(stream).unwrap();
        local.spawn_local(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));

            let mut pos = [0i32; 3];
            loop {
                tokio::select! {

                    _ = interval.tick() => {
                        stream.write_all(b"hello").await.unwrap();
                    }
                    
                    result = stream.read_exact(infinirust::misc::as_bytes_mut(&mut pos)) => {
                        result.unwrap();
                        let mut world = world.lock().unwrap();
                        world.write_chunk(&pos, &mut stream).await;
                    }

                }
            }
        });

        local.await;
    });
}
