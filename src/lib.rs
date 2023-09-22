#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

// extern crate rlibc;
extern crate alloc; // heap allocation/deallocation crate

use core::panic::PanicInfo;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer;

/// allocation失敗時に呼び出されるハンドラ
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub fn init() {
    // Global Descriptor Table()の読み込み
    gdt::init();
    // Interrupt Descriptor Table（割り込み記述子表）を読み込む
    interrupts::init_idt();
    // PICの初期化
    // PICの設定を間違って設定すると未定義動作になるのでunsafe
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        // type_name()で関数名取得
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

// &[&dyn Fn()] : 呼び出せる型の参照 のリスト
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("****************tests start!*************");
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    serial_println!("****************tests finished!*************");
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error:{}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        // 0xf4はisa-debug-exitデバイスのiobase
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
use bootloader::BootInfo;

/// Entry point for `cargo xtest`
#[cfg(test)]
#[no_mangle]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    // use bootloader::BootInfo;

    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
