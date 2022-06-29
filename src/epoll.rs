use std::os::unix::io::{RawFd};
use std::io::Error;
use libc;


pub const READ_FLAG: i32 = libc::EPOLLIN;
pub const READ_ONESHOT_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLIN;
pub const WRITE_FLAG: i32 = libc::EPOLLOUT;
pub const EPOLL_TCP_LISTENER_KEY: u64 = 99;
pub const EPOLL_TLS_LISTENER_KEY: u64 = 100;
pub const EPOLL_DYNKEY: u64 = 101;
pub const EPOLL_MAX_EVENTS: usize = 1024;
pub const EPOLL_MAX_WAIT_TIME: u32 = 5000;


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


pub struct Epoll {
    pub epoll_fd: i32,
    pub dynamic_key: u64,
    pub events: Vec<libc::epoll_event>,
}

impl Epoll {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            epoll_fd: epoll_create()?,
            dynamic_key: EPOLL_DYNKEY,
            events: Vec::with_capacity(EPOLL_MAX_EVENTS),
        })
    }
    pub fn reg_stream(&mut self, stream_fd: i32) {
        self.dynamic_key += 1;
        let r = add_interest(self.epoll_fd, stream_fd,
            libc::epoll_event { events: READ_ONESHOT_FLAGS as u32,
            u64: self.dynamic_key });
        match r {
            Ok(_) => {}, Err(e) => { println!("EPOLL: unable to reg stream: {e}") }
        }
    }
    pub fn reg_listeners(&self, https_listener_fd: i32, http_listener_fd: i32
                         ) -> Result<(), Error> {
        let _ = add_interest(
            self.epoll_fd, https_listener_fd,
            libc::epoll_event {
                events: READ_ONESHOT_FLAGS as u32, u64: EPOLL_TLS_LISTENER_KEY
            }
        )?;
        let _ = add_interest(
            self.epoll_fd, http_listener_fd,
            libc::epoll_event { 
                events: READ_ONESHOT_FLAGS as u32, u64: EPOLL_TCP_LISTENER_KEY
            }
        )?;
        Ok(())
    }
    pub fn wait(&mut self) -> Result<(), Error> {
        self.events.clear();
        let count = syscall!(epoll_wait(
            self.epoll_fd, self.events.as_mut_ptr(),
            EPOLL_MAX_EVENTS as libc::c_int,
            EPOLL_MAX_WAIT_TIME as i32,
        ))? as usize;
        unsafe { self.events.set_len(count) };
        Ok(())
    }
}

fn epoll_create() -> Result<RawFd, Error> {
    let fd = syscall!(epoll_create1(0))?;
    let flags = syscall!(fcntl(fd, libc::F_GETFD))?;
    let _ = syscall!(fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC))?;
    Ok(fd)
}

fn add_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event
                ) -> Result<(), Error> {
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event))?;
    Ok(())
}
