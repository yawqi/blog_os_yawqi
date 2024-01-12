#![no_std]
#![no_main]

use blog_os_yawqi::{exit_qemu, serial_println, QemuExitCode};
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_panic();
    serial_println!("[test not panic as expected]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn should_panic() {
    serial_println!("should_fail... ");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Panic as expected [ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
