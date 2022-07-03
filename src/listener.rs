use std::net::{Ipv4Addr, IpAddr, SocketAddr, TcpListener};
use std::str::FromStr;
use std::os::unix::io::{AsRawFd};
use std::collections::HashMap;
use async_io::{Async};
use async_native_tls::{Identity, TlsAcceptor};
use futures_lite::future;
use crate::conf::CONF;
use crate::epoll;
use crate::spawn::spawn;
use crate::stream_handler::{StreamHandler};
use async_lock::Mutex;
use std::sync::{Arc};


pub struct Listener {
    pub https_listener: Async<TcpListener>,
    pub http_listener: Async<TcpListener>,
    pub tls_acceptor: TlsAcceptor,
    pub epoll: epoll::Epoll,
    pub stream_handlers: HashMap<u64, Arc<Mutex<StreamHandler>>>,
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
            stream_handlers: HashMap::new(),
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
                    self.accept().await;
                } else {
                    spawn(
                        detachable(Arc::clone(&self.stream_handlers[&ev_id]))
                    ).detach();
                }
            }
        }
    }
    pub async fn accept(&mut self) {
        match self.https_listener.accept().await {
            Err(e) => println!("Unable to accept tcp stream: {e}"),
            Ok((https_tcp_stream, _addr)) => {
                match self.tls_acceptor.accept(https_tcp_stream).await {
                    Err(e) => println!("TLS err: {e}"),
                    Ok(tls_stream) => {
                        let fd: i32 = tls_stream.get_ref().as_raw_fd();
                        match self.epoll.reg_tls_stream(fd).await {
                            Err(e) => println!("Epoll err: {e}"),
                            Ok(()) => {
                                let sh = Arc::new(Mutex::new(
                                        StreamHandler::new(tls_stream)
                                ));
                                self.stream_handlers.insert(
                                    self.epoll.tls_stream_id, sh);
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn detachable(q: Arc<Mutex<StreamHandler>>) {
    q.lock().await.process().await;
}

