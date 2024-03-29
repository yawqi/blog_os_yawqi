#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os_yawqi::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use blog_os_yawqi::{
    allocator, hlt_loop, memory, println,
    task::{executor::Executor, keyboard::print_keypresses, Task},
};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

entry_point!(kernel_main);
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello, world!");
    blog_os_yawqi::init();
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut page_frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::new(&boot_info.memory_map) };

    let mut mapper = unsafe { blog_os_yawqi::memory::init(physical_memory_offset) };
    /*
        let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
        unsafe { create_example_mapping(page, &mut mapper, &mut page_frame_allocator) }

        let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
        unsafe {
            page_ptr.offset(400).write_volatile(0xf021_f077_f065_f04e);
        }
    */

    allocator::init_heap(&mut mapper, &mut page_frame_allocator)
        .expect("Create heap memory failed");

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(print_keypresses()));
    executor.run();

    #[cfg(test)]
    test_main();
    println!("It did not crash!");
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

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}
