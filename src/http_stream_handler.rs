use std::io::{ErrorKind};
use async_net::{TcpStream};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use crate::headers::{parse_headers, HeadersParser};


pub struct HttpStreamHandler {
    pub tcp_stream: TcpStream,
    pub buffer: Vec<u8>,
}

impl HttpStreamHandler {
    pub fn new(tcp_stream: TcpStream
    ) -> Self {
        Self {
            tcp_stream: tcp_stream,
            buffer: Vec::<u8>::new(),
        }
    }
    pub async fn process(&mut self) {
        // the only purpose is to redirect all to https
        self.read_headers().await;
        let hp: HeadersParser = parse_headers(&self.buffer);
        if hp.is_valid() == false { return }
        let host = hp.parsed_headers.get("host").unwrap();
        let path = hp.parsed_headers.get("path").unwrap();
        let resp: String = format!("HTTP/1.1 301 Moved Permanently\r\n\
            Location: https://{host}:443{path}\r\n\
            Content-length: 0\r\n\r\n");
        self.write_resp(resp).await;
    }
    pub async fn read_headers(&mut self) {
        let is_oneshot = true;
        self.read(is_oneshot).await
    }
    pub async fn read(&mut self, is_oneshot: bool) {
        let mut buf = [0; 1024*32];
        let mut is_done = false;
        while is_done == false {
            match self.tcp_stream.read(&mut buf).await {
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    println!("Stream read err: {e}");
                }
                Err(e) => {
                    println!("Stream read err: {e}");
                }
                Ok(bytes_read) => {
                    self.buffer.extend_from_slice(&buf);
                    self.buffer = self.buffer[..bytes_read].to_vec();
                    if is_oneshot || bytes_read == 0 { is_done = true; }
                }
            }
        }
    }
    pub async fn write_resp(&mut self, resp: String) {
        let _ = self.tcp_stream.write_all(&resp.as_bytes()).await;
    }
}


