use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PhysFrame, Size4KiB};
use x86_64::PhysAddr;
use x86_64::{
    structures::paging::{OffsetPageTable, PageTable},
    VirtAddr,
};

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// bootloaderのメモリマップから，使用可能な
/// Frameを返すFrameAllocator
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// 渡されたメモリマップからFrameAllocatorを作る
    ///
    /// この関数はunsafe:呼び出し元は渡された
    /// メモリマップが有効であることを保証しなければならない
    /// 特に，USABLEなフレームは実際に未使用でなければならない
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }
    /// メモリマップによって指定されたusableなフレームのイテレータを返す
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // メモリマップからusableな領域を得る
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // それぞれの領域のアドレス範囲にmapで変換する
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // フレームの開始アドレスのイテレータへと変換
        // 4KiBごと
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // 開始アドレスからPhysFrame型を作る
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

/// 常にNoneを返すFrameAllocator
pub struct EmptyFrameAllocator;

// FrameAllocatorはunsafe <- 未使用のフレームのみ取得することを
// 保証しなければならないため
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

/// 与えられたページをフレーム0xb8000に試しにマップしてみる
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    // &mut参照を２度取ろうとしていて未定義動作を起こしかねないためErrorになる
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME : only use test because of unsafe
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

/// 有効なLevel 4 tableへの可変参照を返す。
///
/// この関数はunsafeである：全物理メモリが，渡された
/// `physical_memory_offset`（だけずらしたうえ）で
/// 仮想メモリへとマップされていることを呼び出し元が
/// 保証しなければならない。また，`&mut`参照が複数の
/// 名称を持つこと(mutable aliasingといい，動作が未定義)
/// につながるため，この関数は一度しか呼び出してはならない
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();

    // 開始物理アドレスの取得
    let phys = level_4_page_table.start_address();
    // ページテーブルフレーム(ページのエントリ？)に対応する仮想アドレスの取得
    let virt = physical_memory_offset + phys.as_u64();
    // 仮想アドレスからpage tableの生ポインタを取得
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    return &mut *page_table_ptr;
}

/// 与えられた仮想アドレスを対応する物理アドレスに変換し、
/// そのアドレスがマップされていないなら`None`を返す。
///
/// この関数はunsafeである。なぜなら、呼び出し元は全物理メモリが与えられた
/// `physical_memory_offset`（だけずらした上）でマップされていることを
/// 保証しなくてはならないからである。
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

/// `translate_addr`により呼び出される非公開関数。
///
/// Rustはunsafeな関数の全体をunsafeブロックとして扱ってしまうので、
/// unsafeの範囲を絞るためにこの関数はunsafeにしていない。
/// この関数をモジュール外から呼び出すときは、
/// unsafeな関数`translate_addr`を使って呼び出すこと。
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    // 有効なレベル４フレームをCR3レジスタから読み込む
    let (level_4_page_table, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_page_table;

    // 複数層のページテーブルをたどる
    for &index in &table_indexes {
        // フレームをページテーブルの参照に変換する
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        // Page Table Entryを読んでframeを更新する
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported!"),
        };
    }

    // Page Offsetを足すことで目的の物理アドレスを計算する
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

/// 新しいOffsetPageTableを初期化する。
///
/// この関数はunsafeである：全物理メモリが、渡された
/// `physical_memory_offset`（だけずらしたうえ）で
/// 仮想メモリへとマップされていることを呼び出し元が
/// 保証しなければならない。また、`&mut`参照が複数の
/// 名称を持つこと (mutable aliasingといい、動作が未定義)
/// につながるため、この関数は一度しか呼び出してはならない。
/// 'static はカーネル実行中はずっと生存する
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}
