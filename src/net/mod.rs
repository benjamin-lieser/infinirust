mod packages;
pub use packages::*;
use zerocopy::IntoBytes;

use std::{io::Read, net::TcpStream};

/// Read a String from a sync std::net::TcpStream
pub fn read_string(stream: &mut TcpStream) -> String {
    let mut len = 0u16;
    stream.read_exact(len.as_mut_bytes()).unwrap();
    let mut buffer = vec![0u8; len as usize];
    stream.read_exact(&mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
