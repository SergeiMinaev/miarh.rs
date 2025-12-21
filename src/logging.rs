pub fn format_timestamp() -> String {
    let mut buf = [0u8; 32];
    // SAFETY: localtime_r/strftime are called with valid pointers and buffer size.
    unsafe {
        let mut t = libc::time(std::ptr::null_mut());
        if t == -1 {
            return "unknown-time".to_string();
        }
        let mut tm = std::mem::MaybeUninit::<libc::tm>::uninit();
        if libc::localtime_r(&t as *const libc::time_t, tm.as_mut_ptr()).is_null() {
            return t.to_string();
        }
        let tm = tm.assume_init();
        let fmt = b"%Y-%m-%d %H:%M:%S\0";
        let n = libc::strftime(
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            fmt.as_ptr() as *const libc::c_char,
            &tm,
        );
        if n == 0 {
            return t.to_string();
        }
        String::from_utf8_lossy(&buf[..n as usize]).to_string()
    }
}
