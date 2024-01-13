#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os_yawqi::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os_yawqi::gdt::DOUBLE_FAULT_STACK_INDEX;
use blog_os_yawqi::{exit_qemu, serial_print, serial_println, QemuExitCode};
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_hander)
                .set_stack_index(DOUBLE_FAULT_STACK_INDEX);
        }
        idt
    };
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");
    blog_os_yawqi::gdt::init();
    test_idt_init();
    stack_overflow();
    panic!("Excution after stack overflow!");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os_yawqi::test_panic_handler(info);
}

fn test_idt_init() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_hander(
    _stack: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
