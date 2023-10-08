use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

use infinirust::game::ServerWorld;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn main() -> std::io::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:8042").await.unwrap();
        //common shared state
        let world = Arc::new(Mutex::new(ServerWorld::new(42)));
        // accept connections and process them in a new thread
        for (stream, _) in listener.accept().await {
            let world = world.clone();
            tokio::task::spawn(async {
                handle_connection(stream, world).await;
            });
        }
    });

    Ok(())
}
async fn handle_connection(mut stream: TcpStream, world: Arc<Mutex<ServerWorld>>) {
    let mut pos = [0i32; 3];
    loop {
        stream
            .read_exact(infinirust::misc::as_bytes_mut(&mut pos))
            .await
            .unwrap();
        let mut world = world.lock().unwrap();
        world.write_chunk(&pos, &mut stream).await;
    }
}
