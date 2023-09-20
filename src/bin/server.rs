use std::net::{TcpListener, TcpStream};

use byteorder::{LittleEndian, ReadBytesExt};
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
    let x = stream.read_i32::<LittleEndian>().unwrap();
    let y = stream.read_i32::<LittleEndian>().unwrap();
    let z = stream.read_i32::<LittleEndian>().unwrap();
    let pos = [x, y, z];
    world.write_chunk(&pos, &mut stream);
}
