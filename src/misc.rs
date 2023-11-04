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

pub fn start_server(listen: &str, world_directory: &str) -> std::process::Child {
    std::process::Command::new("cargo")
    .args(["run", "--bin", "server", "--", listen, world_directory])
    .stdin(std::process::Stdio::piped())
    .spawn()
    .expect("Could not start internal server")
}