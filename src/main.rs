#![no_std] // 標準ライブラリにリンクしない
#![no_main] // すべてのRustレベルのエントリポイントを無効にする
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::memory::{self, translate_addr};
// use blog_os::serial_println;
use bootloader::{bootinfo, entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::{structures::paging::Translate, VirtAddr};

pub mod serial;
pub mod vga_buffer;

entry_point!(kernel_main);

// no_mangle -> 名前修飾を無効に
#[no_mangle]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::memory::{BootInfoFrameAllocator, EmptyFrameAllocator};
    use x86_64::{structures::paging::Page, VirtAddr};

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
    // let mut frame_allocator = memory::EmptyFrameAllocator;
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // 未使用のページをマップする
    // address 0を管轄するレベル１テーブルは必ず存在する
    // 存在しないページをマッピングしようとすると
    // EmptyFrameAllocatorはエラーになる(Noneしか返さないので割り当てない)
    // BootInfoFrameAllocatorはUsableなフレームのイテレータを取得する
    let page = Page::containing_address(VirtAddr::new(0xdeadbeef00));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // 新しいマッピングを使って，文字列New!を画面に書き出す
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 Entry: {}: {:?}", i, entry);
    //     }
    // }
    let addresses = [
        // VGAに対応するバッファのページ(恒等対応している)
        0xb8000,
        // コードページのどっか
        0x201008,
        // スタックページのどっか
        0x0100_0020_1a10,
        // 物理アドレス 0 にマップされている仮想アドレス
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

    // Page Tableへのアクセス
    use x86_64::registers::control::Cr3;
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table);

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
