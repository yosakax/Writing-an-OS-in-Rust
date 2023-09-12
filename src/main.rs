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
    // WRITERをロックし続ける

    for _ in 0..vga_buffer::BUFFER_HEIGHT {
        for _ in 0..vga_buffer::BUFFER_WIDTH {
            print!(" ");
        }
        println!();
    }

    println!("HELLO. world{}", "!");
    blog_os::init();

    // // 意図的にカーネル外のメモリにアクセスしてpage faultを起こす
    // let ptr = 0x206c3b as *mut u8;
    // // コードページから読み込む→これはできる
    // unsafe {
    //     let x = *ptr;
    // }
    // println!("read worked!");
    // コードページへの書き込みはできないので，ここでPage Faultが起きる
    // unsafe {
    //     *ptr = 42;
    // }
    // println!("write worked!");

    // Page Tableへのアクセス
    use x86_64::registers::control::Cr3;
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table);

    fn stack_overflow() {
        stack_overflow();
    }

    // 意図的にstack overflowを起こす
    // stack_overflow();

    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // }

    // invoke a breakpoint EXCEPTION
    // x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    println!("Please keyboard input!");
    blog_os::hlt_loop();
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
