use std::net::{Ipv4Addr, IpAddr, SocketAddr, TcpListener};
use std::str::FromStr;
use std::os::unix::io::{AsRawFd};
use async_io::{Async};
use async_native_tls::{Identity, TlsAcceptor};
use futures_lite::future;
use once_cell::sync::Lazy;
use crate::conf::CONF;
use crate::epoll;
use crate::spawn::spawn;
use crate::stream_handler;


pub static LISTENER: Lazy<Listener> = Lazy::new(|| { Listener::new() });


pub struct Listener {
    pub https_listener: Async<TcpListener>,
    pub http_listener: Async<TcpListener>,
    pub tls_acceptor: TlsAcceptor,
    pub epoll: epoll::Epoll,
}


impl Listener {
    pub fn new() -> Self {
        future::block_on(Listener::anew())
    }
    pub async fn anew() -> Self {
        let conf = CONF.read().await;
        let https_addr: SocketAddr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from_str(&conf.ip).unwrap()), conf.https_port
        );
        let http_addr: SocketAddr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from_str(&conf.ip).unwrap()), conf.http_port
        );
        let tls_identity = Identity::from_pkcs12(
            include_bytes!("../identity.pfx"), "").unwrap();
        Self {
            https_listener: Async::<TcpListener>::bind(https_addr).unwrap(),
            http_listener: Async::<TcpListener>::bind(http_addr).unwrap(),
            tls_acceptor: TlsAcceptor::from(
                native_tls::TlsAcceptor::new(tls_identity).unwrap()
            ),
            epoll: epoll::Epoll::new().unwrap(),
        }
    }
    pub async fn main_loop(&self) {
        let https_fd: i32 = self.https_listener.as_raw_fd().clone();
        let http_fd: i32 = self.http_listener.as_raw_fd().clone();
        self.epoll.reg_listeners(https_fd, http_fd).unwrap();
        loop {
            for ev in &self.epoll.events {
                spawn(stream_handler::take_event(*ev)).detach();
            }
        }
    }
}
