#![feature(never_type)]
#![no_std]
#![no_main]

use user::*;

use kernel_api::OsResult;

#[no_mangle]
fn main() {
    let result = main_inner();
    if let Err(error) = result {
        println!("Terminating with error: {:?}", error);
    }
}

fn main_inner() -> OsResult<!> {
    // Lab 5 3
    unimplemented!("main_inner()")
}
