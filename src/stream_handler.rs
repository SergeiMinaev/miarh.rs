use std::io::{ErrorKind};
use std::fs::File;
use std::io::Read;
use async_net::TcpStream;
use async_native_tls::{TlsStream};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use crate::headers::{parse_request, Request};


pub struct StreamHandler {
    pub epoll_ev_id: u64,
    pub epoll_fd: i32,
    pub tls_stream: TlsStream<TcpStream>,
    pub buffer: Vec<u8>,
}

impl StreamHandler {
    pub fn new(epoll_fd: i32, epoll_ev_id: u64,
               tls_stream: TlsStream<TcpStream>) -> Self {
        Self {
            epoll_ev_id: epoll_ev_id,
            epoll_fd: epoll_fd,
            tls_stream: tls_stream,
            buffer: Vec::<u8>::new(),
        }
    }
    pub async fn process(&mut self) {
        println!("Process start...");
        self.read_headers().await;
        let req: Request = parse_request(&self.buffer);
        if req.is_valid() == false { return }
        //self.return_static_test().await;
        self.return_html_test().await;
        println!("Process end...");
    }
    pub async fn read_headers(&mut self) {
        let is_oneshot = true;
        self.read(is_oneshot).await
    }
    pub fn is_headers_valid(&self) -> bool { true }
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
                    if bytes_read == 0 {
                        is_done = true;
                    }
                    self.buffer.extend_from_slice(&buf);
                    self.buffer = self.buffer[..bytes_read].to_vec();
                    if is_oneshot {
                        is_done = true;
                    }
                }
            }
        }
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
        let i: i32 = content.len().try_into().unwrap();
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
        if true {
            let _ = self.tls_stream.write_all(&response).await;
        } else {
            let mut writed_bytes_num: i32 = 0;
            while response.is_empty() == false {
                match self.tls_stream.write(&response).await {
                    Ok(0) => {
                        println!("Err: failed to write whole buffer");
                        break;
                    },
                    Ok(n) => {
                        let nn: i32 = n.try_into().unwrap();
                        writed_bytes_num += nn;
                        response = (&response[n..]).to_vec();
                        println!("Bytes writed: {n}, bytes left: {}.",
                                 i - writed_bytes_num);
                    },
                    Err(e) => {
                        println!("Err while writing bytes: {e}");
                        break;
                    }
                }
            }
        }

    }
}
