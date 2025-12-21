use std::fs::OpenOptions;
use std::io::Write;
use std::panic;
use futures_lite::future;
use miarh::listener::Listener;
use miarh::logging::format_timestamp;


fn main() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let ts = format_timestamp();
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/var/log/miarh_panic.log")
        {
            let _ = writeln!(f, "{ts} panic: {info}");
            let bt = std::backtrace::Backtrace::capture();
            let _ = writeln!(f, "{bt}");
        }
        default_hook(info);
    }));

    let mut listener = Listener::new();
    future::block_on(listener.main_loop());
}
