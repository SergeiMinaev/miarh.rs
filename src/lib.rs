#![feature(io_error_more)]
pub mod cache;
pub mod compress;
pub mod conf;
pub mod epoll;
pub mod headers;
pub mod http;
pub mod http_stream_handler;
pub mod listener;
pub mod multipart;
pub mod mime;
pub mod spawn;
pub mod static_handler;
pub mod stream_handler;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
