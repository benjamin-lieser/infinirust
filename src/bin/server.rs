use std::{net::{TcpListener, TcpStream}, io::Read};

use infinirust::game::ServerWorld;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8042")?;

    let mut world = ServerWorld::new(42);

    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_request(stream?, &mut world);
    }
    Ok(())
}

fn handle_request(mut stream: TcpStream, world: &mut ServerWorld) {
    let mut pos = [0i32;3];
    stream.read_exact(infinirust::misc::as_bytes_mut(&mut pos)).unwrap();
    world.write_chunk(&pos, &mut stream);
}
