#![no_std]
#![no_main]
use user::*;

use kernel_api::syscall;

fn fib(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}
use crate::alloc::vec;
use crate::alloc::vec::Vec;

#[no_mangle]
fn main() {
    let pid = syscall::getpid();
    let beg = syscall::time();
    println!("[{:02}] Started: {:?}", pid, beg);
    let mut v : Vec<u64> = vec!();
    for i in 1..=40 {
        v.push(fib(i));
    }
    let rtn = v.last().expect("push failed");
    let end = syscall::time();
    println!("[{:02}] Result: {} ({:?})", pid, rtn, end - beg);
}
