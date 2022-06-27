use std::os::unix::io::{RawFd};
use std::io::Error;
use libc;
use lazy_static::lazy_static;
use async_lock::{RwLock};


pub const READ_FLAG: i32 = libc::EPOLLIN;
pub const READ_ONESHOT_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLIN;
pub const WRITE_FLAG: i32 = libc::EPOLLOUT;
pub const EPOLL_TCP_LISTENER_KEY: u64 = 99;
pub const EPOLL_TLS_LISTENER_KEY: u64 = 100;
pub const EPOLL_MAX_EVENTS: u32 = 1024;
pub const EPOLL_MAX_WAIT_TIME: u32 = 5000;


lazy_static! {
    pub static ref EPOLL_PARAMS: RwLock<EpollParams> = RwLock::new(epoll_params());
}


#[allow(unused_macros)]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}


pub struct EpollParams {
    pub epoll_fd: i32,
    pub dynamic_key: u64,
}


pub fn epoll_params() -> EpollParams {
    EpollParams {
        epoll_fd: epoll_create().unwrap(),
        dynamic_key: 101,
    }
}

pub fn epoll_create() -> Result<RawFd, Error> {
    let fd = syscall!(epoll_create1(0))?;
    if let Ok(flags) = syscall!(fcntl(fd, libc::F_GETFD)) {
        let _ = syscall!(fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC));
    }
    Ok(fd)
}
