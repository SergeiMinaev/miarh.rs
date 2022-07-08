use std::cmp::min;


const MAX_HEADERS_SIZE: usize = 2048;


pub fn process_headers(buffer: &Vec<u8>) -> bool {
    let max_headers_size = min(MAX_HEADERS_SIZE, buffer.len());
    let mut start = 0;
    let mut is_rn_found = false;
    for mut i in 0..max_headers_size {
        if buffer[i] == b'\r' && buffer[i+1] == b'\n' {
            is_rn_found = true;
            match std::str::from_utf8(&buffer[start..i]) {
                Ok(_line) => {
                    // parse header line here
                },
                Err(_e) => {
                    return false;
                }
            }
            i += 1;
            start = i + 1;
        }
    }
    is_rn_found
}
