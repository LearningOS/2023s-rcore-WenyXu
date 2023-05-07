use alloc::vec::Vec;

// Copy `src_ptr` to `dst`
pub fn copy(dst: Vec<&mut [u8]>, src_ptr: usize, size: usize) {
    let mut written = 0;

    assert!(dst.iter().map(|b| b.len()).sum::<usize>() == size);

    for buffer in dst {
        unsafe {
            buffer.copy_from_slice(core::slice::from_raw_parts(
                (src_ptr + written) as *const u8,
                buffer.len(),
            ))
        }
        written += buffer.len();
    }
}