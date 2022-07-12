use std::cmp::min;
use std::collections::HashMap;
use crate::request::Request;


const MAX_HEADERS_SIZE: usize = 2048;



pub struct HeadersParser {
    pub parsed_headers: HashMap<String, String>,
}
impl HeadersParser {
    fn new() -> Self {
        Self {
            parsed_headers: HashMap::new(),
        }
    }
    pub fn get_req(&self) -> Request {
        Request {
            method: self.parsed_headers.get("method").unwrap().to_string(),
            host: self.parsed_headers.get("host").unwrap().to_string(),
            path: self.parsed_headers.get("path").unwrap().to_string(),
        }
    }
    pub fn is_valid(&self) -> bool {
        let h = &self.parsed_headers;
        if h.contains_key("method")
            && h.contains_key("host")
            && h.contains_key("path")
            && h.get("method").unwrap() == "get"
        { return true }
        return false
    }
    pub fn is_static(&self) -> bool { false }
}


pub fn parse_headers(buffer: &Vec<u8>) -> HeadersParser {
    let mut hp = HeadersParser::new();
    let max_headers_size = min(MAX_HEADERS_SIZE, buffer.len());
    let mut start = 0;
    for mut i in 0..max_headers_size {
        if buffer[i] == b'\r' && buffer[i+1] == b'\n' {
            match std::str::from_utf8(&buffer[start..i]) {
                Ok(_line) => {
                    // parse header line here
                    parse_header_line(_line, &mut hp.parsed_headers);
                },
                Err(_e) => {
                    //return Err("Bad utf-8 sequence.");
                }
            }
            i += 1;
            start = i + 1;
        }
    }
    return hp
}

pub fn parse_header_line(line: &str, parsed_headers: &mut HashMap<String, String>) {
    let line = line.to_lowercase();
    if line.starts_with("get ") {
        parse_method_path_protocol(line, parsed_headers);
    } else if line.starts_with("host: ") {
        parse_host(line, parsed_headers);
    }
}

fn parse_method_path_protocol(s: String, r: &mut HashMap<String, String>) {
    let parts: Vec<&str> = s.split(" ").collect();
    if parts.len() != 3 { return };
    let method = parts[0];
    let path = parts[1];
    let protocol = parts[2];
    if method != "get" {
        println!("Unsupported method: {}", method);
        return;
    }
    if protocol != "http/1.1" {
        println!("Unsupported protocol: {}", protocol);
        return;
    }
    r.insert("method".to_string(), method.to_string());
    r.insert("path".to_string(), path.to_string());
    r.insert("protocol".to_string(), protocol.to_string());
}
fn parse_host(s: String, r: &mut HashMap<String, String>) {
    let parts: Vec<&str> = s.split("host: ").collect();
    if parts.len() != 2 && parts.len() != 1 {
        println!("Invalid 'host' line in headers.");
        return;
    };
    let host_port: Vec<&str> = parts[1].split(":").collect();
    let host = host_port[0].to_lowercase();
    r.insert("host".to_string(), host.to_string());
}
