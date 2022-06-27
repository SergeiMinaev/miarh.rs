use futures_lite::future;
use miarh::conf::Conf;
use miarh::server;


fn main() {
    let conf = Conf::new();
    println!("Here we go.\n{conf:?}");
    future::block_on(server::run());
}
