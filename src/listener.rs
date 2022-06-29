use std::net::{Ipv4Addr, IpAddr, SocketAddr, TcpListener};
use std::str::FromStr;
use std::os::unix::io::{AsRawFd};
use async_io::{Async};
use async_native_tls::{Identity, TlsAcceptor};
use futures_lite::future;
use once_cell::sync::Lazy;
use crate::conf::CONF;
use crate::epoll;


pub static LISTENER: Lazy<Listener> = Lazy::new(|| { Listener::new() });


pub struct Listener {
    pub tls_listener: Async<TcpListener>,
    pub tcp_listener: Async<TcpListener>,
    pub tls_acceptor: TlsAcceptor,
}


impl Listener {
    pub fn new() -> Self {
        future::block_on(Listener::anew())
    }
    pub async fn anew() -> Self {
        let conf = CONF.read().await;
        let tls_addr: SocketAddr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from_str(&conf.ip).unwrap()), conf.tls_port
        );
        let tcp_addr: SocketAddr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from_str(&conf.ip).unwrap()), conf.tcp_port
        );
        let tls_identity = Identity::from_pkcs12(
            include_bytes!("../identity.pfx"), "").unwrap();
        Self {
            tls_listener: Async::<TcpListener>::bind(tls_addr).unwrap(),
            tcp_listener: Async::<TcpListener>::bind(tcp_addr).unwrap(),
            tls_acceptor: TlsAcceptor::from(
                native_tls::TlsAcceptor::new(tls_identity).unwrap()
            ),
        }
    }
    pub async fn main_loop(&self) {
        let tls_fd: i32 = self.tls_listener.as_raw_fd().clone();
        let tcp_fd: i32 = self.tcp_listener.as_raw_fd().clone();
        epoll::reg_listeners(tls_fd, tcp_fd).await;
        let mut events = epoll::init_events().await;
        loop {
            events.clear();
            let events_num: usize = epoll::wait(events.as_mut_ptr()).await;
            unsafe { events.set_len(events_num) };
            for _ev in &events {
            }
        }
    }
}
