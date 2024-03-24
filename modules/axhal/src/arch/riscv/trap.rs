use hal::{ContextArgs, TrapFrame, TrapType};
#[cfg(feature = "monolithic")]
use page_table_entry::MappingFlags;

#[cfg(feature = "monolithic")]
use crate::trap::handle_page_fault;

#[cfg(feature = "signal")]
use crate::trap::handle_signal;

#[allow(unused)]
use super::disable_irqs;

#[cfg(feature = "monolithic")]
use super::enable_irqs;

#[cfg(feature = "monolithic")]
use crate::trap::handle_syscall;

fn handle_breakpoint(sepc: &mut usize) {
    // The sepc has been modified in the kernel callback function
    info!("Exception(Breakpoint) @ {:#x} ", sepc);
}

#[no_mangle]
#[allow(unused)]
pub fn riscv_trap_handler(tf: &mut TrapFrame, from_user: bool, trap_type: TrapType) {
    let scause = riscv::register::scause::read();
    #[cfg(feature = "monolithic")]
    // 这里是测例 interrupt 的对应代码，需要记录中断号
    axfs_ramfs::INTERRUPT.lock().record(scause.code());
    match trap_type {
        TrapType::Breakpoint => handle_breakpoint(&mut tf.sepc),
        TrapType::Time(irq_num) => crate::trap::handle_irq_extern(irq_num, from_user),
        #[cfg(feature = "monolithic")]
        TrapType::UserEnvCall => {
            enable_irqs();
            // get system call return value
            // If it call syscall ok after the handle syscall, then the execve and clone syscall need to add or sub 4 to the sepc for the new task manually.
            // So it doesn't call tf.syscall_ok() here.
            tf[ContextArgs::SEPC] += 4;
            let result = handle_syscall(tf[ContextArgs::SYSCALL], tf.args());
            // cx is changed during sys_exec, so we have to call it again
            tf[ContextArgs::RET] = result as usize;
        }
        #[cfg(feature = "monolithic")]
        TrapType::InstructionPageFault(addr) => {
            info!(
                "I page fault from kernel, addr: {:#x} sepc:{:#x} from user: {}",
                addr, tf.sepc, from_user
            );
            if !from_user {
                unimplemented!("I page fault from kernel");
            }
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::EXECUTE);
        }

        #[cfg(feature = "monolithic")]
        TrapType::LoadPageFault(addr) => {
            info!(
                "L page fault from kernel, addr: {:#x} sepc:{:#x}",
                addr, tf.sepc
            );
            if !from_user {
                unimplemented!("L page fault from kernel");
            }
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::READ);
        }

        #[cfg(feature = "monolithic")]
        TrapType::StorePageFault(addr) => {
            info!(
                "S page fault from kernel, addr: {:#x} sepc:{:#x}",
                addr, tf.sepc
            );
            if !from_user {
                unimplemented!("S page fault from kernel");
            }
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::WRITE);
        }

        _ => {
            panic!(
                "Unhandled trap {:?} @ {:#x}:\n{:#x?}",
                scause.cause(),
                tf.sepc,
                tf
            );
        }
    }

    #[cfg(feature = "signal")]
    if from_user {
        handle_signal();
    }

    #[cfg(feature = "monolithic")]
    // 在保证将寄存器都存储好之后，再开启中断
    // 否则此时会因为写入csr寄存器过程中出现中断，导致出现异常
    disable_irqs();
}

#[no_mangle]
#[cfg(feature = "monolithic")]
/// To handle the first time into the user space
///
/// 1. push the given trap frame into the kernel stack
/// 2. go into the user space
///
/// args:
///
/// 1. kernel_sp: the top of the kernel stack
///
/// 2. frame_base: the address of the trap frame which will be pushed into the kernel stack
pub fn first_into_user(kernel_sp: usize, frame_base: usize) {
    // Make sure that all csr registers are stored before enable the interrupt
    disable_irqs();
    super::flush_tlb(None);

    let trap_frame_size = core::mem::size_of::<TrapFrame>();
    let kernel_base = kernel_sp - trap_frame_size;
    unsafe {
        core::arch::asm!(
            r"
            mv      sp, {frame_base}
            .short  0x2452      # fld  fs0, 272(sp). Warn! it is only used in riscv64
            .short  0x24f2      # fld  fs1, 280(sp). Warn! it is only used in riscv64

            mv      t1, {kernel_base}
            LDR     t0, sp, 3
            STR     gp, t1, 3
            mv      gp, t0
            LDR     t0, sp, 4
            STR     tp, t1, 4                   // save supervisor tp. Note that it is stored on the kernel stack rather than in sp, in which case the ID of the currently running CPU should be stored
            mv      tp, t0                      // tp: now it stores the TLS pointer to the corresponding thread
            csrw    sscratch, {kernel_sp}       // put supervisor sp to scratch
            LDR     t0, sp, 33
            LDR     t1, sp, 32
            csrw    sepc, t0
            csrw    sstatus, t1
            POP_GENERAL_REGS
            LDR     sp, sp, 2
            sret
        ",
            frame_base = in(reg) frame_base,
            kernel_sp = in(reg) kernel_sp,
            kernel_base = in(reg) kernel_base,
        );
    };
}
