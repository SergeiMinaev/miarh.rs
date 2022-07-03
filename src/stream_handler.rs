use std::net::{TcpStream};
use std::time::Duration;
use async_io::{Async, Timer};
use async_native_tls::{TlsStream};


pub struct StreamHandler {
    pub tls_stream: TlsStream<Async<TcpStream>>,
}
impl StreamHandler {
    pub fn new(tls_stream: TlsStream<Async<TcpStream>>) -> Self {
        Self {
            tls_stream: tls_stream,
        }
    }
    pub async fn process(&mut self) {
        println!("Process start...");
        Timer::after(Duration::from_secs(5)).await;
        println!("Process end...");
    }
}
