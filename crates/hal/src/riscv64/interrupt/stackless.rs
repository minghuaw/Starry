use core::arch::asm;

use riscv::register::sie;

use super::{kernel_callback, TrapFrame};

use crate::TrapType;

#[no_mangle]
#[percpu::def_percpu]
static KERNEL_RSP: usize = 0;

#[no_mangle]
#[percpu::def_percpu]
static USER_RSP: usize = 0;

include_asm_marcos!();

#[naked]
pub unsafe extern "C" fn trap_vector_base() {
    asm!(
        // 宏定义
        r"
            .align 4
            .altmacro
        
            csrrw   sp, sscratch, sp
            bnez    sp, uservec
            csrr    sp, sscratch

            addi    sp, sp, -{cx_size}
            
            PUSH_GENERAL_REGS
            csrr    t0, sscratch
            SDR     t0, sp, 2
            csrr    t0, sstatus
            SDR     t0, sp, 32
            csrr    t0, sepc
            SDR     t0, sp, 33


            csrw    sscratch, x0

            mv      a0, sp
            mv      a1, 0
            call kernel_callback

            LDR     t0, sp, 33
            csrw    sepc, t0
            LDR     t0, sp, 32
            csrw    sstatus, t0
            POP_GENERAL_REGS
            LDR     sp, sp, 2
            sret
        ",
        cx_size = const core::mem::size_of::<TrapFrame>(),
        options(noreturn)
    )
}

#[naked]
#[no_mangle]
extern "C" fn user_restore(context: *mut TrapFrame) {
    unsafe {
        asm!(
            r"
                .align 4
                .altmacro
            ",
            // 在内核态栈中开一个空间来存储内核态信息
            // 下次发生中断必然会进入中断入口然后恢复这个上下文.
            // 仅保存 Callee-saved regs、gp、tp、ra.
            "   addi    sp, sp, -18*8
                
                sd      sp, 8*1(sp)
                sd      gp, 8*2(sp)
                sd      tp, 8*3(sp)
                sd      s0, 8*4(sp)
                sd      s1, 8*5(sp)
                sd      s2, 8*6(sp)
                sd      s3, 8*7(sp)
                sd      s4, 8*8(sp)
                sd      s5, 8*9(sp)
                sd      s6, 8*10(sp)
                sd      s7, 8*11(sp)
                sd      s8, 8*12(sp)
                sd      s9, 8*13(sp)
                sd      s10, 8*14(sp)
                sd      s11, 8*15(sp)
                sd      a0,  8*16(sp)
                sd      ra,  8*17(sp)
            ",
            // 将栈信息保存到用户栈.
            // a0 是传入的Context, 然后下面会再次恢复 sp 地址.
            "   sd      sp, 8*0(a0)
                csrw    sscratch, a0
                mv      sp, a0
            
                .short  0x2452      # fld  fs0, 272(sp). Warn! it is only used in riscv64
                .short  0x24f2      # fld  fs1, 280(sp). Warn! it is only used in riscv64

                LDR     t0, sp, 33
                csrw    sepc, t0
                LDR     t0, sp, 32
                csrw    sstatus, t0
                POP_GENERAL_REGS
                LDR     sp, sp, 2
                sret
            ",
            options(noreturn)
        )
    }
}

#[naked]
#[no_mangle]
#[allow(named_asm_labels)]
pub unsafe extern "C" fn uservec() {
    asm!(
        r"
        .altmacro
    ",
        // 保存 general registers, 除了 sp
        "
        PUSH_GENERAL_REGS
        csrr    t0, sscratch
        SDR     t0, sp, 2
        csrr    t0, sstatus
        SDR     t0, sp, 32
        csrr    t0, sepc
        SDR     t0, sp, 33


        csrw    sscratch, x0

        .word   0x10813827          # fsd fs0, 272(sp). Warn! it is only used in riscv64
        .word   0x10913c27          # fsd fs1, 280(sp). Warn! it is only used in riscv64

        mv      a0, sp
        ld      sp, 0*8(a0)
        sd      x0, 0*8(a0)
    ",
        // 恢复内核上下文信息, 仅恢复 callee-saved 寄存器和 ra、gp、tp
        "  
        ld      gp, 8*2(sp)
        ld      tp, 8*3(sp)
        ld      s0, 8*4(sp)
        ld      s1, 8*5(sp)
        ld      s2, 8*6(sp)
        ld      s3, 8*7(sp)
        ld      s4, 8*8(sp)
        ld      s5, 8*9(sp)
        ld      s6, 8*10(sp)
        ld      s7, 8*11(sp)
        ld      s8, 8*12(sp)
        ld      s9, 8*13(sp)
        ld      s10, 8*14(sp)
        ld      s11, 8*15(sp)
        ld      ra,  8*17(sp)
        
        ld      sp, 8(sp)
    ",
        // 回收栈
        "   addi sp, sp, 18*8
        ret
    ",
        options(noreturn)
    );
}

/// Return Some(()) if it was interrupt by syscall, otherwise None.
pub fn run_user_task(context: &mut TrapFrame) -> Option<()> {
    user_restore(context);
    match kernel_callback(context, true) {
        TrapType::UserEnvCall => Some(()),
        _ => None,
    }
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
