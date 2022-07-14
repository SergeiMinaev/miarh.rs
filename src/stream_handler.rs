use std::io::{ErrorKind};
use std::fs::File;
use std::io::Read;
use async_net::{TcpStream};
use async_net::unix::{UnixStream};
use async_native_tls::{TlsStream};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use crate::headers::{parse_headers, HeadersParser};
use crate::request::Request;


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
        let mut hp: HeadersParser = parse_headers(&self.buffer);
        hp.check_is_static().await;
        if hp.is_valid() == false { return }
        if hp.is_static {
            if hp.is_static_valid { self.return_static_test().await; }
            return;
        }
        let req: Request = hp.get_req();
        match self.get_resp(req).await {
            Err(e) => println!("{e}"),
            Ok(resp) => self.write_resp(resp).await,
        }
        //self.return_html_test().await;
    }
    pub async fn read_headers(&mut self) {
        let is_oneshot = true;
        self.read(is_oneshot).await
    }
    pub async fn read(&mut self, is_oneshot: bool) {
        let mut buf = [0; 1024*32];
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
                    self.buffer = self.buffer[..bytes_read].to_vec();
                    if is_oneshot || bytes_read == 0 { is_done = true; }
                }
            }
        }
    }
    pub async fn get_resp(&mut self, req: Request) -> Result<Vec<u8>, &str> {
        if let Some(socket_path) = req.app_socket_path().await {
            match UnixStream::connect(&socket_path).await {
                Ok(mut unixstream) => {
                    let encoded: Vec<u8> = bincode::serialize(&req).unwrap();
                    let _ = unixstream.write_all(&encoded).await;
                    let mut resp: Vec<u8> = vec![];
                    let mut buf = [0; 1024*32];
                    loop {
                        match unixstream.read(&mut buf).await {
                            Err(e) => println!("Err reading unixstream: {e}"),
                            Ok(bytes_read) => {
                                if bytes_read == 0 { break; }
                                resp.extend_from_slice(&buf);
                                return Ok(resp);
                            }
                        }
                    }
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
    pub async fn return_static_test(&mut self) {
        let mut f = File::open("./bg.jpg").unwrap();
        let mut buf: Vec<u8> = Vec::new();
        let content: Vec<u8>;
        f.read_to_end(&mut buf).unwrap();
        content = buf;
        let content_len = format!("Content-Length: {}\r\n", content.len());
        let mime_line = "Content-Type: image/jpeg\r\n".to_string();
        let headers = [
            "HTTP/1.1 200 OK\r\n",
            content_len.as_str(),
            mime_line.as_str(),
            "\r\n"
        ];
        let mut response = headers.join("").to_string().into_bytes();
        response.extend(content);
        let _ = self.tls_stream.write_all(&response).await;
    }
}


