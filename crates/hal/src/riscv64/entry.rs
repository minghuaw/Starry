use crate::PAGE_ITEM_COUNT;
use crate::VIRT_ADDR_START;

#[link_section = ".data.boot_page_table"]
static mut PAGE_TABLE: [u64; PAGE_ITEM_COUNT] = {
    let mut arr: [u64; PAGE_ITEM_COUNT] = [0; PAGE_ITEM_COUNT];
    // 初始化页表信息
    // 0x00000000_80000000 -> 0x80000000 (1G)
    // 高半核
    // 0xffffffc0_00000000 -> 0x00000000 (1G)
    // 0xffffffc0_80000000 -> 0x80000000 (1G)

    // arr[0] = PTE::from_addr(0x0000_0000, PTEFlags::VRWX);
    // arr[1] = PTE::from_addr(0x4000_0000, PTEFlags::VRWX);
    arr[2] = (0x80000 << 10) | 0xef;
    arr[0x100] = (0x00000 << 10) | 0xef;
    arr[0x101] = (0x40000 << 10) | 0xef;
    arr[0x102] = (0x80000 << 10) | 0xef;
    arr[0x106] = (0x80000 << 10) | 0xef;
    arr
};

/// 汇编入口函数
///
/// 分配栈 初始化页表信息 并调到rust入口函数
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        // 1. 设置栈信息
        // sp = bootstack + (hartid + 1) * 0x10000
        "
            la      sp, {boot_stack}
            li      t0, {stack_size}
            add     sp, sp, t0              // set boot stack

            li      s0, {virt_addr_start}   // add virtual address
            or      sp, sp, s0
        ",
        // 2. 开启分页模式
        // satp = (8 << 60) | PPN(page_table)
        "
            la      t0, {page_table}
            srli    t0, t0, 12
            li      t1, 8 << 60
            or      t0, t0, t1
            csrw    satp, t0
            sfence.vma
        ",
        // 3. 跳到 rust_entry 函数，绝对路径
        "
            
            la      a2, {entry}
            or      a2, a2, s0
            jalr    a2                      // call rust_entry
        ",
        stack_size = const crate::STACK_SIZE,
        boot_stack = sym crate::BOOT_STACK,
        page_table = sym PAGE_TABLE,
        virt_addr_start = const VIRT_ADDR_START,
        entry = sym super::rust_entry,
        options(noreturn),
    )
}
