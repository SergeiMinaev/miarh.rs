use std::net::{Ipv4Addr, IpAddr, SocketAddr};
use std::str::FromStr;
use std::os::unix::io::{AsRawFd};
use async_net::{TcpListener, TcpStream};
use async_native_tls::{Identity, TlsAcceptor};
use futures_lite::future;
use crate::conf::CONF;
use crate::epoll;
use crate::spawn::spawn;
use crate::stream_handler::{StreamHandler};
use async_lock::Mutex;
use std::sync::{Arc};


pub struct Listener {
    pub https_listener: TcpListener,
    pub http_listener: TcpListener,
    pub tls_acceptor: Arc<Mutex<TlsAcceptor>>,
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
            https_listener: TcpListener::bind(https_addr).await.unwrap(),
            http_listener: TcpListener::bind(http_addr).await.unwrap(),
            epoll: epoll::Epoll::new().unwrap(),
            tls_acceptor: Arc::new(Mutex::new(TlsAcceptor::from(
                native_tls::TlsAcceptor::new(tls_identity).unwrap()
            ))),
        }
    }
    pub async fn main_loop(&mut self) {
        let https_fd: i32 = self.https_listener.as_raw_fd().clone();
        let http_fd: i32 = self.http_listener.as_raw_fd().clone();
        self.epoll.reg_listeners(https_fd, http_fd).unwrap();
        let mut events = epoll::init_events();
        loop {
            let _ = self.epoll.wait(&mut events);
            for ev in &events {
                let ev_id = ev.u64;
                if ev_id == epoll::EPOLL_HTTPS_LISTENER_ID {
                    self.accept_and_process_https().await;
                } else {
                    println!("Unknown event");
                }
            }
        }
    }
    pub async fn accept_and_process_https(&mut self) {
        match self.https_listener.accept().await {
            Err(e) => println!("Unable to accept tcp stream: {e}"),
            Ok((https_tcp_stream, _addr)) => {
                spawn(process_in_bg(
                    https_tcp_stream, Arc::clone(&self.tls_acceptor)
                )).detach();
            }
        }
    }
}

async fn process_in_bg(
    https_tcp_stream: TcpStream, tls_acceptor: Arc<Mutex<TlsAcceptor>>
) {
    let tls_acceptor = tls_acceptor.lock().await;
    match tls_acceptor.accept(https_tcp_stream).await {
        Err(e) => println!("TLS err: {e}"),
        Ok(tls_stream) => {
            let mut handler = StreamHandler::new(tls_stream);
            handler.process().await;
        }
    }
}
