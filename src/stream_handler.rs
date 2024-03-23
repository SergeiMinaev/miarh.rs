use std::io::{ErrorKind};
use async_net::{TcpStream};
use async_net::unix::{UnixStream};
use async_native_tls::{TlsStream};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use crate::headers::{parse_headers, RequestParser};
use miarh_saras_http::Request;
use crate::http;
use crate::multipart::parse_multipart;
use crate::conf::CONF;
use crate::static_handler;


pub struct StreamHandler {
	pub tls_stream: TlsStream<TcpStream>,
	pub buffer: Vec<u8>,
}

impl StreamHandler {
	pub fn new(tls_stream: TlsStream<TcpStream>) -> Self {
		Self {
			tls_stream: tls_stream,
			buffer: Vec::<u8>::new(),
		}
	}
	pub async fn process(&mut self) {
		self.read_headers().await;
		let mut hp: RequestParser = parse_headers(&self.buffer);
		hp.check_is_static().await;
		hp.check_is_multipart().await;
		hp.parse_query();
		if hp.is_valid() == false { return }
		if hp.is_static {
			if hp.is_static_valid {
				self.return_static(hp).await;
			}
			return;
		}
		if hp.method() == "post" || hp.method() == "put" {
			self.read_post_body(&mut hp).await;
		}
		let req: Request = hp.get_req();
		match self.get_resp(req).await {
			Err(e) => println!("{e}"),
			Ok(resp) => self.write_resp(resp).await,
		}
	}
	pub async fn read_headers(&mut self) {
		let is_oneshot = true;
		self.read(is_oneshot, 0).await
	}
	pub async fn read_post_body(&mut self, hp: &mut RequestParser) {
		let body_start = hp.headers_len + 1;
		let body_end = body_start + hp.content_len();
		if self.buffer.len() < body_end {
			let is_oneshot = false;
			let bytes_left = body_end - self.buffer.len();
			self.read(is_oneshot, bytes_left).await;
		}

		hp.body = self.buffer[hp.headers_len+1..].to_vec();

		if hp.is_multipart {
			parse_multipart(hp).await;
		} else {
		  hp.body_string = String::from_utf8(hp.body[..].to_vec()).unwrap();
		}
	}
	pub async fn read(&mut self, is_oneshot: bool, bytes_left: usize) {
		let conf = CONF.read().await;
		let required_buffer_len = self.buffer.len() + bytes_left;
		let mut buf = [0; 1024];
		let mut is_done = false;
		while is_done == false {
			match self.tls_stream.read(&mut buf).await {
				Err(e) if e.kind() == ErrorKind::WouldBlock => {
					println!("Stream read err: {e}");
				}
				Err(e) => {
					println!("Stream read err: {e}");
				}
				Ok(bytes_read) => {
					self.buffer.extend_from_slice(&buf);
					let real_buffer_len = self.buffer.len() - (buf.len() - bytes_read);
					self.buffer = self.buffer[..real_buffer_len].to_vec();
					if is_oneshot || bytes_read == 0 { is_done = true; }
				}
			}
			if self.buffer.len() >= conf.max_request_size_mb * 1024 * 1024 {
				println!("Max request size exceed.");
				self.return_413_entity_too_large().await;
				return;
			}
			if self.buffer.len() == required_buffer_len {
				is_done = true;
			}
		}
	}

	pub async fn app_socket_path(&mut self, host: &String) -> Option<String> {
		let conf = CONF.read().await;
		for srv in &conf.servers {
			for hostname in &srv.hostnames {
				if hostname == host {
					return Some(srv.socket_path.to_string().clone());
				}
			}
		}
		return None;
	}


	pub async fn get_resp(&mut self, req: Request) -> Result<Vec<u8>, &str> {
		if let Some(socket_path) = self.app_socket_path(&req.host).await {
			match UnixStream::connect(&socket_path).await {
				Ok(mut unixstream) => {
					let data : Vec<u8> = bincode::serialize(&req).unwrap();
					let _ = unixstream.write_all(&data).await.unwrap();
					let _ = unixstream.flush().await;
					let _ = unixstream.close().await;
					let mut resp: Vec<u8> = vec![];
					let mut buf = [0; 1024*32];
					loop {
						match unixstream.read(&mut buf).await {
							Err(e) => println!("Err reading unixstream: {e}"),
							Ok(bytes_read) => {
								if bytes_read == 0 { break; }
								resp.extend_from_slice(&buf);
							}
						}
					}
					return Ok(resp);
				},
				Err(e) => {
					println!("Can't connect to app server: {e}");
				}
			}
		}
		Err("Can't get a response.")
	}

	pub async fn write_resp(&mut self, resp: Vec<u8>) {
		let _ = self.tls_stream.write_all(&resp).await;
	}

	pub async fn return_html_test(&mut self) {
		let resp = "HTTP/1.1 200 OK\r\n\
			Content-Length: 12\r\n\
			\r\n\
			Hello, world\
			\r\n\r\n";
		let resp = resp.to_string().into_bytes();
		let _ = self.tls_stream.write_all(&resp).await;
	}

	pub async fn return_static(&mut self, hp: RequestParser) {
		match static_handler::get_static_file(hp).await {
			Some(r) => { let _ = self.tls_stream.write_all(&r).await; },
			None => self.return_404().await,
		};
	}
	pub async fn return_404(&mut self) {
		let r = http::text_resp(404, "Not found".to_string());
		let _ = self.tls_stream.write_all(&r.get_resp().as_bytes()).await;
	}
	pub async fn return_413_entity_too_large(&mut self) {
		let r = http::text_resp(413, "Request entity too large.".to_string());
		let _ = self.tls_stream.write_all(&r.get_resp().as_bytes()).await;
	}
}
