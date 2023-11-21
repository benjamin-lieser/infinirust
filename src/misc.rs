use std::{io::{Write, Read}, net::TcpStream};

use crate::net::read_string;

pub fn as_bytes(data: &[i32]) -> &[u8] {
    let ptr = data.as_ptr();
    unsafe { std::slice::from_raw_parts(ptr.cast(), data.len() * 4) }
}

pub fn as_bytes_mut(data: &mut [i32]) -> &mut [u8] {
    let ptr = data.as_mut_ptr();
    unsafe { std::slice::from_raw_parts_mut(ptr.cast(), data.len() * 4) }
}

/// Only implement for types with repr(C) and where every bit pattern is valid and no padding in the struct
pub unsafe trait AsBytes: Sized {}

unsafe impl AsBytes for u16 {}
unsafe impl AsBytes for u64 {}
unsafe impl AsBytes for usize {}
unsafe impl AsBytes for [i32;3] {}

pub fn cast_bytes_mut<T: AsBytes>(data: &mut T) -> &mut [u8] {
    unsafe {
        ::core::slice::from_raw_parts_mut((data as *mut T) as *mut u8, ::core::mem::size_of::<T>())
    }
}

pub fn cast_bytes<T: AsBytes>(data: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((data as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

pub fn start_server(world_directory: &str) -> (std::process::Child, String) {
    let mut child = std::process::Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--bin",
            "server",
            "--",
            "internal",
            world_directory,
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    
    stdin.write_all(b"bind\n").unwrap();
    stdin.flush().unwrap();
    let mut bind = "".to_owned();
    let mut c = [0u8];
    loop {
        stdout.read_exact(&mut c).unwrap();
        if c[0] == 10 { //Is newline
            break;
        } else {
            bind.push(c[0] as char);
        }
    }
    child.stdin = Some(stdin);
    child.stdout = Some(stdout);

    println!("Bind from stdout:{}", bind);

    (child, bind)

}

pub fn login(bind: &str, username: &str) -> (TcpStream, u64) {
    let mut stream = TcpStream::connect(bind).expect("Could not connect to server");
    //login package
    let len = username.len();
    assert!(len <= u16::MAX as usize);
    stream.write_all(cast_bytes(&0x0001u16)).unwrap();
    stream.write_all(cast_bytes(&(len as u16))).unwrap();
    stream.write_all(username.as_bytes()).unwrap();

    
    let mut answer = 0u16;
    stream.read_exact(cast_bytes_mut(&mut answer)).unwrap();

    let uid = match answer {
        0x00001 => { //Login Failed
            panic!("Login failed:{}", read_string(&mut stream));
        }
        0x00002 => { //Login success
            let mut uid = 0u64;
            stream.read_exact(cast_bytes_mut(&mut uid)).unwrap();
            uid
        }
        _ => {
            panic!("Invalid package");
        }
    };
    (stream, uid)
}