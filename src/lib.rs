pub mod conf;
pub mod epoll;
pub mod spawn;
pub mod listener;
pub mod stream_handler;
pub mod http_stream_handler;
pub mod headers;
pub mod request;
pub mod mime;
pub mod http;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
