.macro SAVE_REGS, from_user
    addi    sp, sp, -{trapframe_size}
    PUSH_GENERAL_REGS

    csrr    t0, sepc
    csrr    t1, sstatus
    csrrw   t2, sscratch, zero          // save sscratch (sp) and zero it
    STR     t0, sp, 33                  // tf.sepc
    STR     t1, sp, 32                  // tf.sstatus
    STR     t2, sp, 2                   // tf.regs.sp
    .word   0x10813827                  // fsd fs0, 272(sp). Warn! it is only used in riscv64
    .word   0x10913c27                  // fsd fs1, 280(sp). Warn! it is only used in riscv64

.if \from_user == 1
    LDR     t1, sp, 3                   // load supervisor gp
    LDR     t0, sp, 4                   // load supervisor tp
    STR     gp, sp, 3                   // save user gp and tp
    STR     tp, sp, 4
    mv      gp, t1
    mv      tp, t0
.endif
.endm

.macro RESTORE_REGS, from_user
.if \from_user == 1
    LDR     t1, sp, 3                   // load user gp and tp
    LDR     t0, sp, 4
    STR     gp, sp, 3                   // save supervisor tp
    STR     tp, sp, 4                   
    mv      gp, t1
    mv      tp, t0
    addi    t0, sp, {trapframe_size}    // put supervisor sp to scratch
    csrw    sscratch, t0
.endif

    LDR     t0, sp, 33
    LDR     t1, sp, 32
    csrw    sepc, t0
    csrw    sstatus, t1
    .short  0x2452                      // fld  fs0, 272(sp). Warn! it is only used in riscv64
    .short  0x24f2                      // fld  fs1, 280(sp). Warn! it is only used in riscv64
    POP_GENERAL_REGS
    LDR     sp, sp, 2                   // load sp from tf.regs.sp
.endm

.section .text
.balign 4
.global trap_vector_base
trap_vector_base:
    // sscratch == 0: trap from S mode
    // sscratch != 0: trap from U mode
    csrrw   sp, sscratch, sp            // switch sscratch and sp
    bnez    sp, .Ltrap_entry_u

    csrr    sp, sscratch                // put supervisor sp back
    j       .Ltrap_entry_s

.Ltrap_entry_s:
    SAVE_REGS 0
    mv      a0, sp
    li      a1, 0
    call    kernel_callback
    RESTORE_REGS 0
    sret

.Ltrap_entry_u:
    SAVE_REGS 1
    mv      a0, sp
    li      a1, 1
    call    kernel_callback
    RESTORE_REGS 1
    sret
