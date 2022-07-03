use std::net::{TcpStream};
use std::time::Duration;
use std::io::Error;
use std::os::unix::io::AsRawFd;
use async_io::{Async, Timer};
use async_native_tls::{TlsStream};
use crate::epoll::rearm_interest;


pub struct StreamHandler {
    pub epoll_ev_id: u64,
    pub epoll_fd: i32,
    pub tls_stream: TlsStream<Async<TcpStream>>,
    pub is_request_parsed: bool,
    pub is_request_valid: bool,
    pub is_response_ready: bool,
}
impl StreamHandler {
    pub fn new(epoll_fd: i32, epoll_ev_id: u64,
               tls_stream: TlsStream<Async<TcpStream>>) -> Self {
        Self {
            epoll_ev_id: epoll_ev_id,
            epoll_fd: epoll_fd,
            tls_stream: tls_stream,
            is_request_parsed: false,
            is_request_valid: false,
            is_response_ready: false,
        }
    }
    pub async fn process(&mut self) {
        println!("Process start...");
        Timer::after(Duration::from_secs(5)).await;
        println!("Process end...");
        let _ = self.rearm_interest();
    }
    pub fn rearm_interest(&self) {
        let _ = rearm_interest(
            self.epoll_fd, self.tls_stream.get_ref().as_raw_fd(), self.epoll_ev_id
        );
    }
}
