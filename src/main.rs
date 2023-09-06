#![no_std] // 標準ライブラリにリンクしない
#![no_main] // すべてのRustレベルのエントリポイントを無効にする

use core::panic::PanicInfo;

static HELLO: &[u8] = b"HEllo wOrld";

// no_mangle -> 名前修飾を無効に
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // リンカはデフォルトで_startという名前の関数を探すので
    // この関数がエントリポイントとなる

    // VGA bufferの位置の生ポインタ
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            // シアン色
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}

// panic時に呼ばれる関数
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
