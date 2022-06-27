use std::net::{Ipv4Addr, IpAddr, SocketAddr, TcpListener};
use std::str::FromStr;
use std::os::unix::io::{AsRawFd};
use async_io::{Async};
use async_native_tls::{Identity, TlsAcceptor};
use async_lock::{RwLock};
use futures_lite::future;
use once_cell::sync::Lazy;
use qpidfile::Pidfile;
use crate::conf::CONF;
use crate::epoll;
use crate::spawn::{spawn};


pub static SERVER: Lazy<RwLock<Server>> = Lazy::new(|| {
    RwLock::new(Server::new())
});


pub async fn run_server() {
    let _pidfile = Pidfile::new("miarh.pid").unwrap();

    let tls_identity = Identity::from_pkcs12(
        include_bytes!("../identity.pfx"), "").unwrap();
    let _tls_acceptor = TlsAcceptor::from(
        native_tls::TlsAcceptor::new(tls_identity).unwrap()
    );

    let server = SERVER.read().await;
    let tls_fd: i32 = server.tls_listener.as_raw_fd().clone();
    let tcp_fd: i32 = server.tcp_listener.as_raw_fd().clone();
    epoll::reg_listeners(tls_fd, tcp_fd).await;
    let mut events = epoll::init_events().await;
    loop {
        events.clear();
        let events_num: usize = epoll::wait(events.as_mut_ptr()).await;
        unsafe { events.set_len(events_num) };
        for ev in &events {
            spawn(process_event(*ev)).detach();
        }
    }
}


pub async fn process_event(ev: libc::epoll_event) {
    let server = SERVER.read().await;
    if ev.u64 == epoll::EPOLL_TLS_LISTENER_KEY {
        // tls listener event
        server.process_tls_listener_event().await;
    } else if ev.u64 == epoll::EPOLL_TCP_LISTENER_KEY {
        // tcp listener event
    } else {
        // stream event
    }
}


pub struct Server {
    tls_listener: Async<TcpListener>,
    tcp_listener: Async<TcpListener>,
}


impl Server {
    pub fn new() -> Self {
        future::block_on(Server::anew())
    }
    pub async fn anew() -> Self {
        let conf = CONF.read().await;
        let tls_addr: SocketAddr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from_str(&conf.ip).unwrap()), conf.tls_port
        );
        let tcp_addr: SocketAddr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from_str(&conf.ip).unwrap()), conf.tcp_port
        );
        Self {
            tls_listener: Async::<TcpListener>::bind(tls_addr).unwrap(),
            tcp_listener: Async::<TcpListener>::bind(tcp_addr).unwrap(),
        }
    }
    async fn process_tls_listener_event(&self) {
        let stream_addr = self.tls_listener.accept().await;
        match stream_addr {
            Ok((_stream, _addr)) => {}
            Err(ref err) => println!("Unable to accept TLS connection: {}", err),
        }
    }
}
