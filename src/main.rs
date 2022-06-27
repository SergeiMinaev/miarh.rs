use futures_lite::future;
use miarh::conf::Conf;
use miarh::server::run_server;


fn main() {
    let conf = Conf::new();
    println!("Here we go.\n{conf:?}");
    future::block_on(run_server());
}
