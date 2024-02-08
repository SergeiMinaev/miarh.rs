use std::io::{Write};
use std::path::Path;
use brotli;
use crate::headers::RequestParser;


pub fn compress(buf: &Vec<u8>) -> Vec<u8> {
    let mut brotli_buf: Vec<u8> = Vec::new();
    let mut writer = brotli::CompressorWriter::new(
        &mut brotli_buf,
        4096,
        11,
        22);
    match writer.write_all(&buf) {
        Err(e) => println!("handle_compress() err: {}", e),
        Ok(_) => {},
    }
    drop(writer);
    brotli_buf
}

pub fn is_compressable(hp: RequestParser) -> bool {
    if hp.is_accept_brotli() == false { return false }
    let path = hp.get_header("static_path");
    let compressable: Vec<&str> = vec!["html", "css", "js"];
    match Path::new(&path).extension() {
        Some(v) => {
            let v = v.to_str().unwrap();
            return compressable.contains(&v.to_lowercase().as_str())
        },
        None => return false
    }
}
