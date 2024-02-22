use std::fs::File;
use std::io::{ ErrorKind, Read };
use crate::cache::{ CACHE };
use crate::headers::{ RequestParser };
use crate::compress;
use crate::mime;


pub async fn get_static_file(hp: RequestParser) -> Option<Vec<u8>> {
	let path = hp.get_header("static_path").split("?").next().unwrap().to_string();
	let mut content: Vec<u8> = vec![];
	let mut content_encoding = "";
	let mime_line = match mime::get_mimetype(&path) {
		None => String::from(""),
		Some(m) => format!("Content-Type: {}\r\n", m),
	};
	if compress::is_compressable(hp) {
		let mut cache = CACHE.write().await;
		content = match cache.get(&path).await {
			None => return None,
			Some(content) => {
				content
			},
		};
		content_encoding = "Content-Encoding: br\r\n";
	} else {
		match File::open(&path) {
		    Ok(mut f) => {
				match f.read_to_end(&mut content) {
					Ok(_) => { },
					Err(e) if e.kind() == ErrorKind::IsADirectory => {
						return None
					},
					Err(_) => {
						return None
					},
				}
			},
		    Err(_) => return None,
		};
	}
	let content_len = format!("Content-Length: {}\r\n", content.len());
	let headers = [
		"HTTP/1.1 200 OK\r\n",
		content_len.as_str(),
		content_encoding,
		mime_line.as_str(),
		"\r\n"
	];
	let mut response = headers.join("").to_string().into_bytes();
	response.extend(content);
	Some(response)
}
