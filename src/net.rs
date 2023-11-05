use std::{net::TcpStream, io::Read};

use crate::misc::cast_bytes_mut;

pub fn read_string(stream: &mut TcpStream) -> String {
    let mut len = 0u16;
    stream.read_exact(cast_bytes_mut(&mut len)).unwrap();
    let mut buffer = vec![0u8;len as usize];
    stream.read_exact(&mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}