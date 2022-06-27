use std::net::{Ipv4Addr, IpAddr, SocketAddr, TcpListener};
use std::str::FromStr;
use async_io::{Async};
use async_native_tls::{Identity, TlsAcceptor};
use qpidfile::Pidfile;
use crate::conf::CONF;


pub async fn run() {
    let _pidfile = Pidfile::new("miarh.pid").unwrap();

    let conf = CONF.read().await;
    let tls_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(&conf.ip).unwrap()), conf.tls_port);
    let tcp_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(&conf.ip).unwrap()), conf.tcp_port);
    let _tls_listener = Async::<TcpListener>::bind(tls_addr).unwrap();
    let _tcp_listener = Async::<TcpListener>::bind(tcp_addr).unwrap();

    let tls_identity = Identity::from_pkcs12(include_bytes!("../identity.pfx"), "").unwrap();
    let _tls_acceptor = TlsAcceptor::from(native_tls::TlsAcceptor::new(tls_identity).unwrap());
}
