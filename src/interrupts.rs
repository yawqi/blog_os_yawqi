use core::panic;

use crate::gdt::DOUBLE_FAULT_STACK_INDEX;
use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_STACK_INDEX);
        }
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("Exception Breakpoint\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("DOUBLE FAULT EXCEPTION\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_interrupt() {
    x86_64::instructions::interrupts::int3();
}
