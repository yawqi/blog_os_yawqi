#![no_std]
#![no_main]
#![allow(dead_code)]

use core::panic::PanicInfo;

mod vga_buffer;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

static HELLO: &[u8] = b"Hello, World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello, world!");
    panic!("Some panic info!");
    loop {}
}
