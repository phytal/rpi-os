use core::fmt::{self, Arguments};
use core::fmt::Write;
use alloc::string::String;
use kernel_api::syscall;
use core::result::Result::*; 
struct SyncConsole;


impl fmt::Write for SyncConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        syscall::write_str(s);

        Ok(())
    }
}


pub fn vprint(s: &str) {
    let mut c = SyncConsole;
    c.write_str(s).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (crate::console::vprint(&crate::alloc::format!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        crate::console::vprint(&crate::alloc::format!("{}\n", &crate::alloc::format!($($arg)*)));
    });
}

struct Console;


impl fmt::Write for Console{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            syscall::write(b);
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! uprint {
    ($($arg:tt)*) => ($crate::console::uvprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! uprintln {
 () => (print!("\n"));
    ($($arg:tt)*) => ({
        $crate::console::uvprint(format_args!($($arg)*));
        $crate::uprint!("\n");
    })
}


pub fn uvprint(args: Arguments) {
    let mut c = Console;
    c.write_fmt(args).unwrap();
}
