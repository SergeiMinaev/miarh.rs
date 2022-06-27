use std::net::{Ipv4Addr, IpAddr, SocketAddr, TcpListener};
use std::str::FromStr;
use std::os::unix::io::{AsRawFd};
use async_io::{Async};
use async_native_tls::{Identity, TlsAcceptor};
use qpidfile::Pidfile;
use crate::conf::CONF;
use crate::epoll;


pub async fn run() {
    let _pidfile = Pidfile::new("miarh.pid").unwrap();

    let conf = CONF.read().await;
    let tls_addr: SocketAddr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from_str(&conf.ip).unwrap()), conf.tls_port
    );
    let tcp_addr: SocketAddr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from_str(&conf.ip).unwrap()), conf.tcp_port
    );
    let tls_listener = Async::<TcpListener>::bind(tls_addr).unwrap();
    let tcp_listener = Async::<TcpListener>::bind(tcp_addr).unwrap();

    let tls_identity = Identity::from_pkcs12(
        include_bytes!("../identity.pfx"), "").unwrap();
    let _tls_acceptor = TlsAcceptor::from(
        native_tls::TlsAcceptor::new(tls_identity).unwrap()
    );

    let tls_fd: i32 = tls_listener.as_raw_fd().clone();
    let tcp_fd: i32 = tcp_listener.as_raw_fd().clone();
    epoll::reg_listeners(tls_fd, tcp_fd).await;
    let mut events = epoll::init_events().await;
    loop {
        events.clear();
        let _events_num: usize = epoll::wait(events.as_mut_ptr()).await;
    }
}
