#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os_yawqi::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os_yawqi::{hlt_loop, print, println};
use core::panic::PanicInfo;
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello, world!");
    blog_os_yawqi::init();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use blog_os_yawqi::test_panic_handler;
    test_panic_handler(info)
}
