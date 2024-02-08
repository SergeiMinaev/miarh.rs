use std::cmp::min;
use std::collections::HashMap;
use std::path::Path;
use cookie::Cookie;
use miarh_saras_http::{ Request, RequestFile };
use crate::conf::CONF;


pub const MAX_HEADERS_SIZE: usize = 2048;


#[derive(Debug)]
pub struct RequestParser {
    pub parsed_headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub is_static: bool,
    pub is_static_valid: bool,
    pub is_multipart: bool,
    pub headers_len: usize,
    pub body: Vec<u8>,
    pub body_string: String,
    pub route: HashMap<String, String>,
    pub files: HashMap<String, RequestFile>,
}
impl RequestParser {
    fn new() -> Self {
        Self {
            parsed_headers: HashMap::new(),
            query: HashMap::new(),
            is_static: false,
            is_static_valid: false,
            is_multipart: false,
            headers_len: 0,
            body: vec![],
            body_string: String::new(),
            route: HashMap::new(),
            files: HashMap::new(),
        }
    }
    pub fn get_req(&mut self) -> Request {
        Request {
            method: self.parsed_headers.get("method").unwrap().to_string(),
            host: self.parsed_headers.get("host").unwrap().to_string(),
            path: self.parsed_headers.get("path").unwrap().to_string(),
            session_id: self.parsed_headers.get("session_id")
                .unwrap_or(&"".to_string()).to_string(),
            query: self.query.clone(),
            //body: body,
            body_string: self.body_string.clone(),
            route: self.route.clone(),
            files: self.files.clone(),
        }
    }
    pub fn get_header(&self, name: &str) -> String {
        let h = &self.parsed_headers;
        match h.get(name) {
            Some(v) => v.to_string(),
            None => "".to_string(),
        }
    }
    pub fn is_valid(&self) -> bool {
        let h = &self.parsed_headers;
        if h.contains_key("method")
            && h.contains_key("host")
            && h.contains_key("path")
            && (h.get("method").unwrap() == "get"
                || h.get("method").unwrap() == "post"
                || h.get("method").unwrap() == "put"
                || h.get("method").unwrap() == "delete")
            && { self.is_static_valid == true || self.is_static == false }
        { return true }
        return false
    }
    pub fn is_accept_brotli(&self) -> bool {
        let h = &self.parsed_headers;
        if h.contains_key("accept-encoding")
                && h.get("accept-encoding").unwrap().contains("br") {
            return true }
        return false;
    }
    pub async fn check_is_multipart(&mut self) {
        self.is_multipart = self.parsed_headers.contains_key("content-type")
            && self.parsed_headers.get("content-type").unwrap()
            .contains("multipart/form-data");
    }
    pub async fn check_is_static(&mut self) {
        if self.parsed_headers.contains_key("host")
                && self.parsed_headers.contains_key("path") {
            self._check_is_static().await;
        } else {
            self.remove_trailing_slash();
            self.is_static = false; self.is_static_valid = false; return;
        }
    }
    pub async fn _check_is_static(&mut self) {
        let conf = CONF.read().await;
        let path = self.parsed_headers.get("path").unwrap();
        if path.starts_with("/static/")
                || path == &conf.index_url
                || path.starts_with(&conf.acme_challenge_url) {
            self.is_static = true
        } else {
            self.remove_trailing_slash();
            self.is_static = false; self.is_static_valid = false; return;
        }
        let headers_host = self.parsed_headers.get("host").unwrap();
        for srv_conf in &conf.servers {
            for host in &srv_conf.hostnames {
                if host == headers_host {
                    let static_dir = Path::new(&srv_conf.static_dir);
                    if path.starts_with("/static/") {
                        let path = path.replace("/static/", "");
                        let full_path = static_dir.join(path);
                        if full_path.starts_with(&srv_conf.static_dir) {
                            self.is_static_valid = true;
                            self.parsed_headers.insert(
                                "static_path".to_string(),
                                full_path.display().to_string()
                            );
                            return;
                        }
                    } else if path == &conf.index_url {
                        self.is_static_valid = true;
                        self.parsed_headers.insert(
                            "static_path".to_string(),
                            srv_conf.index_path.to_string()
                        );
                        return;
                    } else if path.starts_with(&conf.acme_challenge_url) {
                        let path = path.replace(&conf.acme_challenge_url, "");
                        let acme_dir = Path::new(&conf.acme_challenge_dir);
                        let full_path = acme_dir.join(path);
                        if full_path.starts_with(&conf.acme_challenge_dir) {
                            self.is_static_valid = true;
                            self.parsed_headers.insert(
                                "static_path".to_string(),
                                full_path.display().to_string()
                            );
                            return;
                        }
                    }
                    break;
                }
            }
        }
    }
    pub async fn is_acme(path: &String) -> bool {
        let conf = CONF.read().await;
        return path.starts_with(&conf.acme_challenge_url);
    }
    pub fn method(&mut self) -> &String {
        return self.parsed_headers.get("method").unwrap()
    }
    pub fn content_len(&mut self) -> usize {
        let s = self.parsed_headers.get("content-length").unwrap();
        match s.parse::<usize>() {
            Err(_) => 0,
            Ok(v) => v,
        }
    }
    pub fn remove_trailing_slash(&mut self) {
        let mut path = self.parsed_headers.get("path").unwrap().to_string();
        if path.ends_with("/") {
            path.pop();
            *self.parsed_headers.get_mut("path").unwrap() = path;
        }
    }
    pub fn parse_query(&mut self) {
        match self.parsed_headers.get("path").unwrap().split("?").take(2).nth(1) {
            None => { return },
            Some(q) => {
                for kv in q.split("&") {
                    let mut kv_it = kv.split("=").take(2);
                    match (kv_it.next(), kv_it.next()) {
                        (Some(k), Some(v)) => {
                            self.query.insert(k.to_string(), v.to_string());
                        },
                        _ => continue
                    }
                }
            }
        }
    }
}


pub fn parse_headers(buffer: &Vec<u8>) -> RequestParser {
    let mut hp = RequestParser::new();
    let max_headers_size = min(MAX_HEADERS_SIZE, buffer.len());
    let mut start = 0;
    for mut i in 0..max_headers_size {
        if buffer[i] == b'\r' && buffer[i+1] == b'\n' {
            match std::str::from_utf8(&buffer[start..i]) {
                Ok(_line) => {
                    parse_header_line(_line, &mut hp.parsed_headers);
                },
                Err(_e) => {
                    println!("Bad utf-8 sequence.");
                }
            }
            i += 1;
            start = i + 1;
        }
        hp.headers_len = i;
        // stop at end of headers
        if i >= 3 && buffer[i-3] == b'\r' && buffer[i-2] == b'\n'
            && buffer[i-1] == b'\r' && buffer[i] == b'\n'
        {
            return hp;
        }
    }
    return hp
}

pub fn parse_header_line(line: &str, parsed_headers: &mut HashMap<String, String>) {
    let lowerline = line.to_lowercase();
    if lowerline.starts_with("get ")
            || lowerline.starts_with("post ") 
            || lowerline.starts_with("delete ") 
            || lowerline.starts_with("put ") {
        parse_method_path_protocol(line, parsed_headers);
    } else if lowerline.starts_with("host: ") {
        parse_host(lowerline, parsed_headers);
    } else if lowerline.starts_with("content-length: ") {
        parse_content_len(lowerline, parsed_headers);
    } else if lowerline.starts_with("content-type: ") {
        parse_content_type(lowerline, parsed_headers);
    } else if lowerline.starts_with("accept-encoding: ") {
        parse_accept_encoding(lowerline, parsed_headers);
    } else if lowerline.starts_with("cookie: ") {
        parse_cookies(&lowerline, parsed_headers);
    }
}

fn parse_method_path_protocol(s: &str, r: &mut HashMap<String, String>) {
    let parts: Vec<&str> = s.split(" ").collect();
    if parts.len() != 3 { return };
    let method = parts[0];
    let method = method.to_lowercase();
    let path = parts[1];
    let protocol = parts[2];
    let protocol = protocol.to_lowercase();
    if method != "get" && method != "post" && method != "put" && method != "delete" {
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
fn parse_content_len(s: String, r: &mut HashMap<String, String>) {
    let parts: Vec<&str> = s.split(" ").collect();
    if parts.len() != 2 {
        println!("Invalid 'content-length' header.");
        return;
    }
    let len = parts[1];
    r.insert("content-length".to_string(), len.to_string());
}
fn parse_accept_encoding(s: String, r: &mut HashMap<String, String>) {
    let parts: Vec<&str> = s.split("accept-encoding: ").collect();
    if parts.len() != 2 {
        println!("Invalid 'accept-encoding' header.");
        return;
    }
    let v = parts[1];
    r.insert("accept-encoding".to_string(), v.to_string());
}

fn parse_cookies(s: &str, r: &mut HashMap<String, String>) {
    let parts: Vec<&str> = s.split("cookie: ").collect();
    if parts.len() != 2 {
        println!("Invalid 'cookie' header.");
        return;
    }
    let cookie = parts[1];
    if let Ok(c) = Cookie::parse(cookie) {
        let (name, value) = c.name_value();
        if name == "session_id".to_string() && value.len() < 100 {
            r.insert("session_id".to_string(), value.to_string());
        }
    }
}
fn parse_content_type(s: String, r: &mut HashMap<String, String>) {
    let parts: Vec<&str> = s.split("content-type: ").collect();
    let v = parts[1];
    r.insert("content-type".to_string(), v.to_string());
}
