use std::io::{Read, ErrorKind};
use std::fs::File;
use async_net::{TcpStream};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use crate::headers::{parse_headers, RequestParser};
use crate::http;


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
        let mut hp: RequestParser = parse_headers(&self.buffer);
        hp.check_is_static().await;
        if hp.is_valid() == false { return }
        let host = hp.parsed_headers.get("host").unwrap();
        let path = hp.parsed_headers.get("path").unwrap();
        if hp.is_static && hp.is_static_valid  && RequestParser::is_acme(&path).await {
            self.return_static(hp).await;
            return;
        }
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
    pub async fn return_static(&mut self, hp: RequestParser) {
        let path = hp.get_header("static_path");
        let mut f = match File::open(&path) {
            Ok(f) => f,
            Err(_) => {
                println!("File not found: {path}");
                self.return_404().await;
                return;
            },
        };
        let mut buf: Vec<u8> = Vec::new();
        match f.read_to_end(&mut buf) {
            Ok(_) => {},
            Err(e) if e.kind() == ErrorKind::IsADirectory => {
                println!("{path} is a directory.");
                self.return_404().await;
                return;
            },
            Err(e) => {
                println!("return_static() err: {e}");
                self.return_404().await;
                return;
            },
        }
        let content: Vec<u8>;
        let mime_line = "Content-Type: text/html\r\n";
        content = buf;
        let content_len = format!("Content-Length: {}\r\n", content.len());
        let headers = [
            "HTTP/1.1 200 OK\r\n",
            content_len.as_str(),
            mime_line,
            "\r\n"
        ];
        let mut response = headers.join("").to_string().into_bytes();
        response.extend(content);
        let _ = self.tcp_stream.write_all(&response).await;
    }
    pub async fn return_404(&mut self) {
        let r = http::text_resp(404, "Not found".to_string());
        let _ = self.tcp_stream.write_all(&r.get_resp().as_bytes()).await;
    }
}


