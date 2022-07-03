use futures_lite::future;
use miarh::conf::Conf;
use miarh::listener::{Listener};


fn main() {
    let mut listener = Listener::new();
    future::block_on(listener.main_loop());
}
