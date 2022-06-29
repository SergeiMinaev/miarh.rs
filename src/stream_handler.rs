use std::net::{TcpStream};
use async_io::{Async};
use crate::epoll;
use crate::listener::LISTENER;


pub async fn take_event(ev: libc::epoll_event) {
    let mut handler = StreamHandler::new(ev);
    handler.handle().await;
}


pub struct StreamHandler {
    main_ev: libc::epoll_event,
    tcp_stream: Option<Async<TcpStream>>,
}

impl StreamHandler {
    pub fn new(ev: libc::epoll_event) -> Self {
        Self {
            main_ev: ev,
            tcp_stream: None,
        }
    }
    pub async fn handle(&mut self) {
        if self.main_ev.u64 == epoll::EPOLL_TLS_LISTENER_KEY {
            println!("HTTPS listener event");
            self.handle_https_listener_ev().await;
        } else {
            println!("Stream event");
        }
    }
    pub async fn handle_https_listener_ev(&mut self) {
        self.get_tcp_stream().await;
    }
    pub async fn get_tcp_stream(&mut self) {
        match LISTENER.https_listener.accept().await {
            Ok((tcp_stream, _addr)) => {
                self.tcp_stream = Some(tcp_stream);
            },
            Err(ref e) => println!("https_listener.accept() failed: {}", e),
        }
    }
}
