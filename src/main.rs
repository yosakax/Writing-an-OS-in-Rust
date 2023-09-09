#![no_std] // 標準ライブラリにリンクしない
#![no_main] // すべてのRustレベルのエントリポイントを無効にする
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

// use blog_os::serial_println;
use core::panic::PanicInfo;

pub mod serial;
pub mod vga_buffer;

// no_mangle -> 名前修飾を無効に
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // リンカはデフォルトで_startという名前の関数を探すので
    // この関数がエントリポイントとなる

    println!("HELLO. world{}", "!");
    // panic!("some panic message");

    #[cfg(test)]
    test_main();

    loop {}
}

// panic時に呼ばれる関数
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    loop {}
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
