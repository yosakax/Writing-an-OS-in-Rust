#![no_std] // 標準ライブラリにリンクしない
#![no_main] // すべてのRustレベルのエントリポイントを無効にする

use core::panic::PanicInfo;

mod vga_buffer;

static HELLO: &[u8] = b"HEllo wOrld";

// no_mangle -> 名前修飾を無効に
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // リンカはデフォルトで_startという名前の関数を探すので
    // この関数がエントリポイントとなる

    println!("HELLO. world{}", "!");
    panic!("some panic message");

    loop {}
}

// panic時に呼ばれる関数
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
