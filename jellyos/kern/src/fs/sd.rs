use core::time::Duration;
use shim::io;

use fat32::traits::BlockDevice;

extern "C" {
    /// A global representing the last SD controller error that occured.
    static sd_err: i64; // TODO: process this

    /// Initializes the SD card controller.
    ///
    /// Returns 0 if initialization is successful. If initialization fails,
    /// returns -1 if a timeout occured, or -2 if an error sending commands to
    /// the SD controller occured.
    fn sd_init() -> i32;

    /// Reads sector `n` (512 bytes) from the SD card and writes it to `buffer`.
    /// It is undefined behavior if `buffer` does not point to at least 512
    /// bytes of memory. Also, the caller of this function should make sure that
    /// `buffer` is at least 4-byte aligned.
    ///
    /// On success, returns the number of bytes read: a positive number.
    ///
    /// On error, returns 0. The true error code is stored in the `sd_err`
    /// global. `sd_err` will be set to -1 if a timeout occured or -2 if an
    /// error sending commands to the SD controller occured. Other error codes
    /// are also possible but defined only as being less than zero.
    fn sd_readsector(n: i32, buffer: *mut u8) -> i32;
}


use pi::timer;

use crate::console::kprintln;
#[no_mangle]
fn wait_micros(us : u32) {
    timer::spin_sleep(Duration::from_micros(us as u64));
}

/// A handle to an SD card controller.
#[derive(Debug)]
pub struct Sd;

impl Sd {
    /// Initializes the SD card controller and returns a handle to it.
    /// The caller should assure that the method is invoked only once during the
    /// kernel initialization. We can enforce the requirement in safe Rust code
    /// with atomic memory access, but we can't use it yet since we haven't
    /// written the memory management unit (MMU).
    pub unsafe fn new() -> Result<Sd, io::Error> {
        match sd_init() {
            0 => {
                return Ok(Sd);
            },
            -1 => {
                kprintln!("sdcard err: {}", sd_err);
                Err(io::Error::new(io::ErrorKind::TimedOut, "SD Card initialization timed out"))
            },
            -2 => {
                kprintln!("sdcard err: {}", sd_err);
                Err(io::Error::new(io::ErrorKind::ConnectionRefused, "SD Card did not recieve commands"))
            },
            _ => {
                kprintln!("sdcard err: {}", sd_err);
                Err(io::Error::new(io::ErrorKind::Uncategorized, "Unknown error"))
            }
        }
    }
}

impl BlockDevice for Sd {
    /// Reads sector `n` from the SD card into `buf`. On success, the number of
    /// bytes read is returned.
    ///
    /// # Errors
    ///
    /// An I/O error of kind `InvalidInput` is returned if `buf.len() < 512` or
    /// `n > 2^31 - 1` (the maximum value for an `i32`).
    ///
    /// An error of kind `TimedOut` is returned if a timeout occurs while
    /// reading from the SD card.
    ///
    /// An error of kind `Other` is returned for all other errors.
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = unsafe { sd_readsector(n.try_into().unwrap(), buf as *mut [u8] as *mut u8) };
        Ok(bytes_read.try_into().unwrap())
    }

    fn write_sector(&mut self, _n: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("SD card and file system are read only")
    }
}
