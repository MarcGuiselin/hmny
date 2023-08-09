#[no_mangle]
pub extern "C" fn signal(input_ptr: u64, input_length: u64) -> u64 {
    // Parse input object
    let slice =
        unsafe { std::slice::from_raw_parts_mut(input_ptr as *mut u8, input_length as usize) };

    // Do something with the input object
    let mut index = 0;
    slice.iter_mut().for_each(|value| {
        index += 1;
        *value += index;
    });

    // Return length of output
    slice.len() as u64
}
