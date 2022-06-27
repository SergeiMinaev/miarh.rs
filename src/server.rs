use std::net::{Ipv4Addr, IpAddr, SocketAddr, TcpListener};
use std::str::FromStr;
use async_io::{Async};
use qpidfile::Pidfile;
use crate::conf::CONF;


pub async fn run() {
    let _pidfile = Pidfile::new("miarh.pid").unwrap();

    let conf = CONF.read().await;
    let tls_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(&conf.ip).unwrap()), conf.tls_port);
    let _tls_listener = Async::<TcpListener>::bind(tls_addr).unwrap();
}
