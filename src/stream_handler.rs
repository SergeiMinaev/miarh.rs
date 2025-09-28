use std::io::{ErrorKind};
use std::time::Duration;
use async_net::{TcpStream};
use async_net::unix::{UnixStream};
use async_native_tls::{TlsStream};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use futures_lite::future::race;
use async_io::Timer;
use futures_lite::io::{split, copy};
use crate::headers::{parse_headers, RequestParser};
use miarh_saras_http::Request;
use crate::http;
use crate::multipart::parse_multipart;
use crate::conf::CONF;
use crate::static_handler;


pub struct StreamHandler {
	pub tls_stream: Option<TlsStream<TcpStream>>,
	pub buffer: Vec<u8>,
}

impl StreamHandler {
	pub fn new(tls_stream: TlsStream<TcpStream>) -> Self {
		Self {
			tls_stream: Some(tls_stream),
			buffer: Vec::<u8>::new(),
		}
	}
    pub async fn process(&mut self) {
        let conf = CONF.read().await;
        loop {
            self.buffer.clear();
			let timeout = Timer::after(Duration::from_secs(conf.keep_alive_timeout_sec));
			if race(
				self.read_headers(),
				async { timeout.await; Err(()) }
			).await.is_err() {
				//println!("Keep-Alive timeout reached, closing connection");
				break;
			}

            let mut hp: RequestParser = parse_headers(&self.buffer);
            hp.check_is_static().await;
            hp.check_is_multipart().await;
            hp.check_is_keep_alive().await;
            hp.parse_query();
            if hp.is_websocket_upgrade() {
                //println!("WS upgrade detected: host='{}' path='{}' headers_len={}", hp.get_header("host"), hp.get_header("path"), hp.headers_len);
                if let Some(ws_socket) = self.app_ws_socket_path(&hp.get_header("host")).await {
                    //println!("Proxying WS to backend '{}'", ws_socket);
                    self.proxy_websocket(&hp, &ws_socket).await;
                    println!("Finished WS proxying for host='{}' path='{}'", hp.get_header("host"), hp.get_header("path"));
                    break;
                } else {
                    println!("No ws_backend configured for host '{}'", hp.get_header("host"));
                    self.return_404().await;
                    break;
                }
            }
            if !hp.is_valid() {
                break;
            }

			let keep_alive = hp.is_keep_alive;
            if hp.is_static {
                if hp.is_static_valid {
                    self.return_static(hp).await;
                }
                if !keep_alive { break; }
                continue;
            }

            if hp.method() == "post" || hp.method() == "put" {
                self.read_post_body(&mut hp).await;
            }
            
            let req: Request = hp.get_req();
            match self.get_resp(req).await {
                Err(e) => println!("{e}"),
                Ok(resp) => self.write_resp(resp).await,
            }
            
            if !keep_alive { break; }
        }
    }
	async fn read_headers(&mut self) -> Result<(), ()> {
        let is_oneshot = true;
        self.read(is_oneshot, 0).await;
        Ok(())
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
        while !is_done {
            match self.tls_stream.as_mut().unwrap().read(&mut buf).await {
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    println!("Stream read err: {e}");
                }
                Err(e) => {
                    println!("Stream read err: {e}");
                    return;
                }
                Ok(bytes_read) => {
                    if bytes_read == 0 { break; }
                    self.buffer.extend_from_slice(&buf[..bytes_read]);
                    if is_oneshot || self.buffer.len() == required_buffer_len {
                        is_done = true;
                    }
                }
            }
            if self.buffer.len() >= conf.max_request_size_mb * 1024 * 1024 {
                println!("Max request size exceed.");
                self.return_413_entity_too_large().await;
                return;
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

	pub async fn app_ws_socket_path(&mut self, host: &String) -> Option<String> {
		let conf = CONF.read().await;
		for srv in &conf.servers {
			for hostname in &srv.hostnames {
				if hostname == host {
					if let Some(ws) = &srv.ws_backend {
						return Some(ws.to_string().clone());
					}
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
					// let mut buf = [0; 1024*32];
					let mut buf = vec![0; 1024 * 32].into_boxed_slice();
					loop {
						match unixstream.read(&mut buf).await {
							Err(e) => println!("Err reading unixstream: {e}"),
							Ok(bytes_read) => {
								if bytes_read == 0 { break; }
								resp.extend_from_slice(&buf[..bytes_read]);
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

	pub async fn proxy_websocket(&mut self, hp: &RequestParser, backend: &String) {
		// for now only support unix: backend (format: unix:/path/to.sock)
		if backend.starts_with("unix:") {
			let path = backend.trim_start_matches("unix:");
			//println!("proxy_websocket: connecting to unix socket '{}'", path);
			match UnixStream::connect(path).await {
				Ok(mut backend_stream) => {
					//println!("proxy_websocket: connected to backend");
					// forward buffered headers and any extra buffered bytes
					let header_bytes = &self.buffer[..=hp.headers_len];
					if let Err(e) = backend_stream.write_all(header_bytes).await {
						//println!("proxy_websocket: error writing headers to backend: {e}");
						self.return_404().await;
						return;
					}
					//println!("proxy_websocket: forwarded {} header bytes", header_bytes.len());
					if self.buffer.len() > hp.headers_len + 1 {
						let start = hp.headers_len + 1;
						let extra = &self.buffer[start..];
						if let Err(e) = backend_stream.write_all(extra).await {
							//println!("proxy_websocket: error writing extra buffered bytes to backend: {e}");
							self.return_404().await;
							return;
						}
						//println!("proxy_websocket: forwarded {} extra bytes already buffered", extra.len());
					}
					// take client stream out of self (once)
					let client_stream = self.tls_stream.take().unwrap();
					// replace the polling loop with a single bidirectional copy that
					// efficiently transfers data in both directions and returns when
					// either side closes or an error occurs.
					//println!("proxy_websocket: starting bidirectional copy client<->backend");

					// Split each stream into independent read/write halves and run two
					// concurrent copy futures. This avoids async mutex deadlocks.
					let (mut c_reader, mut c_writer) = split(client_stream);
					let (mut b_reader, mut b_writer) = split(backend_stream);

					let c2b = async {
						match copy(&mut c_reader, &mut b_writer).await {
							Ok(n) => {
								//println!("proxy_websocket: client->backend copied {} bytes", n);
								Ok::<(), std::io::Error>(())
							}
							Err(e) => Err(e),
						}
					};

					let b2c = async {
						match copy(&mut b_reader, &mut c_writer).await {
							Ok(n) => {
								//println!("proxy_websocket: backend->client copied {} bytes", n);
								Ok::<(), std::io::Error>(())
							}
							Err(e) => Err(e),
						}
					};

					match race(c2b, b2c).await {
						Ok(_) => {
							//println!("proxy_websocket: one direction finished");
						}
						Err(e) => {
							//println!("proxy_websocket: tunnel error: {e}");
						}
					}

					// attempt graceful shutdown of both writers
					let _ = c_writer.close().await;
					let _ = b_writer.close().await;
					//println!("proxy_websocket: tunnel ended, streams closed");
				},
				Err(e) => {
					println!("Can't connect to ws backend: {e}");
					self.return_404().await;
				}
			}
		} else {
			println!("Unsupported ws backend: {}", backend);
			self.return_404().await;
		}
	}

	pub async fn write_resp(&mut self, resp: Vec<u8>) {
		let _ = self.tls_stream.as_mut().unwrap().write_all(&resp).await;
		// Без этого при больших ответах иногда бывает NS_ERROR_NET_PARTIAL_TRANSFER (в браузере).
		let _ = self.tls_stream.as_mut().unwrap().flush().await;
	}

	pub async fn return_html_test(&mut self) {
		let resp = "HTTP/1.1 200 OK\r\n\
			Content-Length: 12\r\n\
			\r\n\
			Hello, world\
			\r\n\r\n";
		let resp = resp.to_string().into_bytes();
		let _ = self.tls_stream.as_mut().unwrap().write_all(&resp).await;
	}

	pub async fn return_static(&mut self, hp: RequestParser) {
		match static_handler::get_static_file(hp).await {
			Some(r) => { let _ = self.tls_stream.as_mut().unwrap().write_all(&r).await; },
			None => self.return_404().await,
		};
	}
	pub async fn return_404(&mut self) {
		let r = http::text_resp(404, "Not found".to_string());
		let _ = self.tls_stream.as_mut().unwrap().write_all(&r.get_resp().as_bytes()).await;
	}
	pub async fn return_413_entity_too_large(&mut self) {
		let r = http::text_resp(413, "Request entity too large.".to_string());
		let _ = self.tls_stream.as_mut().unwrap().write_all(&r.get_resp().as_bytes()).await;
	}
}
