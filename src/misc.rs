pub fn as_bytes(data : &[i32]) -> &[u8] {
    let ptr = data.as_ptr();
    unsafe {
        std::slice::from_raw_parts(ptr.cast(), data.len() * 4)
    }
}

pub fn as_bytes_mut(data : &mut [i32]) -> &mut [u8] {
    let ptr = data.as_mut_ptr();
    unsafe {
        std::slice::from_raw_parts_mut(ptr.cast(), data.len() * 4)
    }
}

pub fn first_none<T>(data : &[Option<T>]) -> Option<usize> {
    for (idx, x) in data.iter().enumerate() {
        match x {
            None => {
                return Some(idx);
            }
            Some(_) => {}
        }
    }
    return None
}