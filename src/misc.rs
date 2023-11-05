use std::io::{Write, Read};

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

pub fn cast_bytes_mut<T: AsBytes>(data: &mut T) -> &mut [u8] {
    unsafe {
        ::core::slice::from_raw_parts_mut((data as *mut T) as *mut u8, ::core::mem::size_of::<T>())
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
