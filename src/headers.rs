use std::cmp::min;
use std::collections::HashMap;


const MAX_HEADERS_SIZE: usize = 2048;


pub struct Request {
    pub parsed_headers: HashMap<String, String>,
}
impl Request {
    fn new() -> Self {
        Self {
            parsed_headers: HashMap::new(),
        }
    }
    pub fn is_valid(&self) -> bool {
        if self.parsed_headers.contains_key("method")
            && self.parsed_headers.get("method").unwrap() == "get"
        {
            return true
        }
        return false
    }
}


pub fn parse_request(buffer: &Vec<u8>) -> Request {
    let mut req = Request::new();
    let max_headers_size = min(MAX_HEADERS_SIZE, buffer.len());
    let mut start = 0;
    for mut i in 0..max_headers_size {
        if buffer[i] == b'\r' && buffer[i+1] == b'\n' {
            match std::str::from_utf8(&buffer[start..i]) {
                Ok(_line) => {
                    // parse header line here
                    parse_header_line(_line, &mut req.parsed_headers);
                },
                Err(_e) => {
                    //return Err("Bad utf-8 sequence.");
                }
            }
            i += 1;
            start = i + 1;
        }
    }
    req
}

pub fn parse_header_line(line: &str, parsed_headers: &mut HashMap<String, String>) {
    let line = line.to_lowercase();
    if line.starts_with("get ") {
        parsed_headers.insert("method".to_string(), "get".to_string());
    }
}
