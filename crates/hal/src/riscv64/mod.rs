mod boards;
mod consts;
mod entry;
mod interrupt;
// mod page_table;

mod sbi;
mod timer;
// use alloc::vec::Vec;
pub use boards::*;
pub use consts::*;

// use fdt::Fdt;
pub use interrupt::*;
// pub use page_table::*;

use riscv::register::sstatus;
pub use sbi::*;
pub use timer::*;

#[no_mangle]
extern "C" fn rust_entry(hartid: usize, device_tree: usize) {
    crate::clear_bss();
    // ArchInterface::init_logging();
    crate::init_logging();
    // Init allocator

    #[cfg(feature = "alloc")]
    crate::init_allocator();

    // allocator::init();
    let smp: usize = option_env!("AX_SMP").unwrap_or("").parse().unwrap_or(1);
    percpu::init(smp);
    percpu::set_local_thread_pointer(hartid);

    let (hartid, _device_tree) = boards::init_device(hartid, device_tree);

    // let mut dt_buf = Vec::new();

    // if device_tree != 0 {
    //     let fdt = unsafe { Fdt::from_ptr(device_tree as *const u8).unwrap() };

    //     dt_buf.extend_from_slice(unsafe {
    //         core::slice::from_raw_parts(device_tree as *const u8, fdt.total_size())
    //     });

    //     info!("There has {} CPU(s)", fdt.cpus().count());

    //     fdt.memory().regions().for_each(|x| {
    //         info!(
    //             "memory region {:#X} - {:#X}",
    //             x.starting_address as usize,
    //             x.starting_address as usize + x.size.unwrap()
    //         );

    //         crate::add_memory_region(
    //             x.starting_address as usize | VIRT_ADDR_START,
    //             (x.starting_address as usize + x.size.unwrap()) | VIRT_ADDR_START,
    //         );
    //     });
    // }

    crate::prepare_drivers();

    // if let Ok(fdt) = Fdt::new(&dt_buf) {
    //     for node in fdt.all_nodes() {
    //         crate::try_to_add_device(&node);
    //     }
    // }

    // 开启 SUM
    unsafe {
        // 开启浮点运算
        sstatus::set_fs(sstatus::FS::Dirty);
    }

    // drop(dt_buf);

    // To enable the interrupt
    init_interrupt();
    // crate::ArchInterface::main(hartid);
    crate::kernel_main(hartid);
    shutdown();
}

#[inline]
pub fn wfi() {
    unsafe {
        riscv::register::sstatus::clear_sie();
        riscv::asm::wfi();
        riscv::register::sstatus::set_sie();
    }
}
