#![no_std] // 標準ライブラリにリンクしない
#![no_main] // すべてのRustレベルのエントリポイントを無効にする
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use blog_os::memory::{self};
use blog_os::task::executor::Executor;
use blog_os::task::keyboard;
use blog_os::task::{simple_executor::SimpleExecutor, Task};
// use blog_os::serial_println;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

pub mod serial;
pub mod vga_buffer;

entry_point!(kernel_main);

// no_mangle -> 名前修飾を無効に
#[no_mangle]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::allocator;
    use blog_os::memory::BootInfoFrameAllocator;
    use x86_64::VirtAddr;

    // リンカはデフォルトで_startという名前の関数を探すので
    // この関数がエントリポイントとなる
    // WRITERをロックし続ける

    // window flush!
    for _ in 0..vga_buffer::BUFFER_HEIGHT {
        for _ in 0..vga_buffer::BUFFER_WIDTH {
            print!(" ");
        }
        println!();
    }

    println!("HELLO. world{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    println!("Please keyboard input!");
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number : {}", number);
}

/// panic時に呼ばれる関数
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
