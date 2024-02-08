#![feature(io_error_more)]
pub mod compress;
pub mod conf;
pub mod epoll;
pub mod spawn;
pub mod listener;
pub mod stream_handler;
pub mod http_stream_handler;
pub mod headers;
pub mod mime;
pub mod http;
pub mod multipart;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
