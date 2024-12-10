use core::{fmt, str};
use core::fmt::Write;
use core::time::Duration;

use crate::*;

macro_rules! err_or {
    ($ecode:expr, $rtn:expr) => {{
        let e = OsError::from($ecode);
        if let OsError::Ok = e {
            Ok($rtn)
        } else {
            Err(e)
        }
    }};
}
use core::arch::asm;

pub fn sleep(span: Duration) -> OsResult<Duration> {
    if span.as_millis() > core::u64::MAX as u128 {
        panic!("too big!");
    }

    let ms = span.as_millis() as u64;
    let mut ecode: u64;
    let mut elapsed_ms: u64;

    unsafe {
        asm!(
            "mov x0, {ms}",
            "svc {nr_sleep}",
            "mov {elapsed_ms}, x0",
            "mov {ecode}, x7",
            ms = in(reg) ms,
            nr_sleep = const NR_SLEEP,
            elapsed_ms = out(reg) elapsed_ms,
            ecode = out(reg) ecode,
            out("x0") _,   // Clobbers x0
            out("x7") _,   // Clobbers x7
            options(nostack),
        );
    }

    err_or!(ecode, Duration::from_millis(elapsed_ms))
}

pub fn time() -> Duration {
    let mut ecode: u64;
    let mut current_time: u64;
    let mut frac_time: u64;

    unsafe {
        asm!(
            "svc {nr_time}",
            "mov {current_time}, x0",
            "mov {frac_time}, x1",
            "mov {ecode}, x7",
            nr_time = const NR_TIME,
            current_time = out(reg) current_time,
            frac_time = out(reg) frac_time,
            ecode = out(reg) ecode,
        );
    }

    let _ = OsError::from(ecode);
    Duration::from_secs(current_time) + Duration::from_nanos(frac_time)
}

pub fn exit() -> ! {
    unsafe {
        asm!(
            "svc {nr_exit}",
            nr_exit = const NR_EXIT,
            options(nostack),
        );
    }
    loop {}
}

pub fn write(b: u8) {
    let mut ecode: u64;

    unsafe {
        asm!(
            "mov w0, {b:w}",
            "svc {nr_write}",
            "mov {ecode}, x7",
            b = in(reg) b,
            nr_write = const NR_WRITE,
            ecode = out(reg) ecode,
            out("x7") _,   // Clobbers x0
            options(nostack),
        );
    }

    let _ = OsError::from(ecode);
}

pub fn write_str(msg: &str) {
    let mut ecode: u64;
    let mut printed_len: u64;

    unsafe {
        asm!(
            "mov x0, {str_addr:x}",
            "mov x1, {str_len:x}",
            "svc {nr_write_str}",
            "mov {printed_len:x}, x0",
            "mov {ecode:x}, x7",
            str_addr = in(reg) msg as *const str as *const usize as usize,
            str_len = in(reg) msg.len(),
            nr_write_str = const NR_WRITE_STR,
            ecode = out(reg) ecode,
            printed_len = out(reg) printed_len,
            out("x0") _,   // Clobbers x0
            out("x7") _,   // Clobbers x0
            options(nostack),
        );
    }
    assert!(msg.len() == printed_len as usize); // why does this fail if i don't include??

    let _ = OsError::from(ecode);

}

pub fn getpid() -> u64 {
    let mut ecode: u64;
    let mut pid: u64;

    unsafe {
        asm!(
            "svc {nr_getpid}",
            "mov {pid}, x0",
            "mov {ecode}, x7",
            nr_getpid = const NR_GETPID,
            pid = out(reg) pid,
            ecode = out(reg) ecode,
            out("x0") _,   // Clobbers x0
            out("x7") _,   // Clobbers x0
            options(nostack),
        );
    }

    let _ = OsError::from(ecode);

    pid
}

pub fn sock_create() -> SocketDescriptor {
    // Lab 5 2.D
    unimplemented!("sock_create")
}

pub fn sock_status(descriptor: SocketDescriptor) -> OsResult<SocketStatus> {
    // Lab 5 2.D
    unimplemented!("sock_status")
}

pub fn sock_connect(descriptor: SocketDescriptor, addr: IpAddr) -> OsResult<()> {
    // Lab 5 2.D
    unimplemented!("sock_connect")
}

pub fn sock_listen(descriptor: SocketDescriptor, local_port: u16) -> OsResult<()> {
    // Lab 5 2.D
    unimplemented!("sock_listen")
}

pub fn sock_send(descriptor: SocketDescriptor, buf: &[u8]) -> OsResult<usize> {
    // Lab 5 2.D
    unimplemented!("sock_send")
}

pub fn sock_recv(descriptor: SocketDescriptor, buf: &mut [u8]) -> OsResult<usize> {
    // Lab 5 2.D
    unimplemented!("sock_recv")
}


