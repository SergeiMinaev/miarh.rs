use std::os::unix::io::{RawFd};
use std::io::Error;
use libc;
use async_lock::{RwLock};
use once_cell::sync::Lazy;


pub const READ_FLAG: i32 = libc::EPOLLIN;
pub const READ_ONESHOT_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLIN;
pub const WRITE_FLAG: i32 = libc::EPOLLOUT;
pub const EPOLL_TCP_LISTENER_KEY: u64 = 99;
pub const EPOLL_TLS_LISTENER_KEY: u64 = 100;
pub const EPOLL_MAX_EVENTS: usize = 1024;
pub const EPOLL_MAX_WAIT_TIME: u32 = 5000;


static EPOLL_PARAMS: Lazy<RwLock<EpollParams>> = Lazy::new(|| {
    RwLock::new(epoll_params())
});


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

pub async fn init_events() -> Vec<libc::epoll_event> {
    Vec::with_capacity(EPOLL_MAX_EVENTS)
}

fn add_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event
                ) -> Result<(), Error> {
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event))?;
    Ok(())
}

pub async fn reg_listeners(tls_listener_fd: i32, tcp_listener_fd: i32) {
    let params = EPOLL_PARAMS.read().await;
    let _ = add_interest(
        params.epoll_fd, tls_listener_fd,
        libc::epoll_event { events: READ_ONESHOT_FLAGS as u32, u64: EPOLL_TLS_LISTENER_KEY }
    ).unwrap();
    let _ = add_interest(
        params.epoll_fd, tcp_listener_fd,
        libc::epoll_event { events: READ_ONESHOT_FLAGS as u32, u64: EPOLL_TCP_LISTENER_KEY }
    ).unwrap();
}

pub async fn wait(events: *mut libc::epoll_event) -> usize {
    let params = EPOLL_PARAMS.read().await;
    syscall!(epoll_wait(
        params.epoll_fd, events,
        EPOLL_MAX_EVENTS as libc::c_int,
        EPOLL_MAX_WAIT_TIME as i32,
    )).unwrap() as usize
}
