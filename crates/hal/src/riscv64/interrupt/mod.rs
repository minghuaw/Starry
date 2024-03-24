#[macro_use]
mod macros;
mod frame;
cfg_if::cfg_if! {
    if #[cfg(feature = "stackful")] {
        mod stackful;
    } else  {
        mod stackless;
        pub use stackless::*;
    }
}

pub use frame::TrapFrame;

use crate::{kernel_interrupt, TrapType, VIRT_ADDR_START};
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval,
};
/// 内核中断回调
#[no_mangle]
fn kernel_callback(context: &mut TrapFrame, from_user: bool) -> TrapType {
    let scause = scause::read();
    let stval = stval::read();
    debug!(
        "中断发生: {:#x} {:?}  stval {:#x}  sepc: {:#x}",
        scause.bits(),
        scause.cause(),
        stval,
        context.sepc
    );
    let trap_type = match scause.cause() {
        // 中断异常
        Trap::Exception(Exception::Breakpoint) => {
            // QUESTION: Why do we need to add 2 to sepc?
            context.sepc += 2;
            TrapType::Breakpoint
        }
        Trap::Exception(Exception::LoadFault) => {
            if stval > VIRT_ADDR_START {
                panic!("kernel error: {:#x}", stval);
            }
            TrapType::Unknown
        }
        Trap::Exception(Exception::UserEnvCall) => TrapType::UserEnvCall,
        // 时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => TrapType::Time(scause.bits()),
        Trap::Exception(Exception::StorePageFault) => TrapType::StorePageFault(stval),
        Trap::Exception(Exception::InstructionPageFault) => TrapType::InstructionPageFault(stval),
        Trap::Exception(Exception::IllegalInstruction) => TrapType::IllegalInstruction(stval),
        Trap::Exception(Exception::LoadPageFault) => TrapType::LoadPageFault(stval),
        Trap::Interrupt(Interrupt::SupervisorExternal) => TrapType::SupervisorExternal,
        _ => {
            error!(
                "内核态中断发生: {:#x} {:?}  stval {:#x}  sepc: {:#x}",
                scause.bits(),
                scause.cause(),
                stval,
                context.sepc
            );
            panic!("未知中断: {:#x?}", context);
        }
    };
    kernel_interrupt(context, from_user, trap_type);
    // crate::api::ArchInterface::kernel_interrupt(context, from_user, trap_type);
    trap_type
}

#[allow(dead_code)]
#[inline(always)]
pub fn enable_irq() {
    unsafe {
        sie::set_sext();
        sie::set_ssoft();
    }
}

#[inline(always)]
pub fn enable_external_irq() {
    unsafe {
        sie::set_sext();
    }
}

/// 设置中断
pub fn init_interrupt() {
    // crate::riscv64::page_table::sigtrx::init();
    // 输出内核信息

    extern "C" {
        fn trap_vector_base();
    }
    unsafe {
        core::arch::asm!("csrw stvec, a0", in("a0") trap_vector_base as usize);

        // 测试
        info!("测试 ebreak exception");
        core::arch::asm!("ebreak");
    }

    // // 初始化定时器
    // crate::riscv64::timer::init();
}

/// Create a trampoline for sigreturn
///
/// This page can be accessed by user space
///
/// To use this function, you need to add the page to the linker script like this after the text section:
/// ```
/// . = ALIGN(4K);
/// *(.text.signal_trampoline)
/// . = ALIGN(4K);
/// ```
///
#[naked]
#[no_mangle]
#[link_section = ".text.signal_trampoline"]
unsafe extern "C" fn _sigreturn() -> ! {
    core::arch::asm!(
        // 1. 设置栈信息
        // sp = bootstack + (hartid + 1) * 0x10000
        "
            li  a7, 139
            ecall
        ",
        options(noreturn)
    )
}

/// To get the address of sigreturn trampoline
#[allow(dead_code)]
pub fn get_sigreturn() -> usize {
    _sigreturn as usize
}
