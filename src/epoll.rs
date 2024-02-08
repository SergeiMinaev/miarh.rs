use std::os::unix::io::{RawFd};
use std::io::Error;
use libc;


pub const READ_FLAG: i32 = libc::EPOLLIN;
pub const READ_ONESHOT_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLIN;
pub const WRITE_FLAG: i32 = libc::EPOLLOUT;
pub const EPOLL_MAX_EVENTS: usize = 1024;
pub const EPOLL_MAX_WAIT_TIME: u32 = 5000;

pub const EPOLL_HTTPS_LISTENER_ID: u64 = 0;
pub const EPOLL_HTTP_LISTENER_ID: u64 = 1;
pub const EPOLL_HTTPS_TCP_STREAM_START_ID: u64 = 18_000_000_000_000_000_000;
pub const EPOLL_TLS_STREAM_START_ID: u64 = 10_000_000_000_000_000_000;


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
    pub tls_stream_id: u64,
}

impl Epoll {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            epoll_fd: epoll_create()?,
            tls_stream_id: EPOLL_TLS_STREAM_START_ID,
        })
    }
    pub fn reg_listeners(&self, https_listener_fd: i32, http_listener_fd: i32
                         ) -> Result<(), Error> {
        let _ = add_interest(
            self.epoll_fd, https_listener_fd,
            libc::epoll_event {
                events: READ_FLAG as u32, u64: EPOLL_HTTPS_LISTENER_ID
            }
        )?;
        let _ = add_interest(
            self.epoll_fd, http_listener_fd,
            libc::epoll_event { 
                events: READ_FLAG as u32, u64: EPOLL_HTTP_LISTENER_ID
            }
        )?;
        Ok(())
    }
    pub fn reg_tls_stream(&mut self, stream_fd: i32) -> Result<(), Error> {
        self.tls_stream_id += 1;
        self.reg_stream(stream_fd, self.tls_stream_id)?;
        Ok(())
    }
    pub fn reg_stream(&mut self, stream_fd: i32, id: u64) -> Result<(), Error> {
        add_interest(self.epoll_fd, stream_fd,
            libc::epoll_event { events: READ_ONESHOT_FLAGS as u32,
            u64: id}
        )?;
        Ok(())
    }
    pub fn wait(&self, events: &mut Vec<libc::epoll_event>) -> Result<(), Error> {
        events.clear();
        let count = syscall!(epoll_wait(
            self.epoll_fd, events.as_mut_ptr(),
            EPOLL_MAX_EVENTS as libc::c_int,
            EPOLL_MAX_WAIT_TIME as i32,
        ))? as usize;
        unsafe { events.set_len(count) };
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

pub fn rearm_interest(epoll_fd: RawFd, fd: RawFd, ev_id: u64) -> Result<(), Error> {
    let mut ev = libc::epoll_event { events: READ_ONESHOT_FLAGS as u32, u64: ev_id };
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_MOD, fd, &mut ev))?;
    Ok(())
}

pub fn init_events() -> Vec<libc::epoll_event> {
    Vec::with_capacity(EPOLL_MAX_EVENTS)
}
