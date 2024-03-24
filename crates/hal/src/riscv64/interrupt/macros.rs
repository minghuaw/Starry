/// define the macro for the assembly code

macro_rules! include_asm_marcos {
    () => {
        #[cfg(target_arch = "riscv32")]
        core::arch::global_asm!(
            r"
        .ifndef XLENB
        .equ XLENB, 4

        .macro LDR rd, rs, off
            lw \rd, \off*XLENB(\rs)
        .endm
        .macro STR rs2, rs1, off
            sw \rs2, \off*XLENB(\rs1)
        .endm

        .endif"
        );

        #[cfg(target_arch = "riscv64")]
        core::arch::global_asm!(
            r"
        .ifndef XLENB
        .equ XLENB, 8

        .macro LDR rd, rs, off
            ld \rd, \off*XLENB(\rs)
        .endm
        .macro STR rs2, rs1, off
            sd \rs2, \off*XLENB(\rs1)
        .endm

        .endif",
        );

        #[cfg(feature = "stackful")]
        core::arch::global_asm!(
            r"
        .ifndef .LPUSH_POP_GENERAL_REGS
        .equ .LPUSH_POP_GENERAL_REGS, 0

        .macro PUSH_POP_GENERAL_REGS, op
            \op ra, sp, 1
            \op t0, sp, 5
            \op t1, sp, 6
            \op t2, sp, 7
            \op s0, sp, 8
            \op s1, sp, 9
            \op a0, sp, 10
            \op a1, sp, 11
            \op a2, sp, 12
            \op a3, sp, 13
            \op a4, sp, 14
            \op a5, sp, 15
            \op a6, sp, 16
            \op a7, sp, 17
            \op s2, sp, 18
            \op s3, sp, 19
            \op s4, sp, 20
            \op s5, sp, 21
            \op s6, sp, 22
            \op s7, sp, 23
            \op s8, sp, 24
            \op s9, sp, 25
            \op s10, sp, 26
            \op s11, sp, 27
            \op t3, sp, 28
            \op t4, sp, 29
            \op t5, sp, 30
            \op t6, sp, 31
        .endm

        .macro PUSH_GENERAL_REGS
            PUSH_POP_GENERAL_REGS STR
        .endm
        .macro POP_GENERAL_REGS
            PUSH_POP_GENERAL_REGS LDR
        .endm

        .endif"
        );

        #[cfg(not(feature = "stackful"))]
        // need to save the gp and the tp register
        core::arch::global_asm!(
            r"
        .ifndef .LPUSH_POP_GENERAL_REGS
        .equ .LPUSH_POP_GENERAL_REGS, 0

        .macro PUSH_POP_GENERAL_REGS, op
            \op ra, sp, 1
            \op gp, sp, 3
            \op tp, sp, 4
            \op t0, sp, 5
            \op t1, sp, 6
            \op t2, sp, 7
            \op s0, sp, 8
            \op s1, sp, 9
            \op a0, sp, 10
            \op a1, sp, 11
            \op a2, sp, 12
            \op a3, sp, 13
            \op a4, sp, 14
            \op a5, sp, 15
            \op a6, sp, 16
            \op a7, sp, 17
            \op s2, sp, 18
            \op s3, sp, 19
            \op s4, sp, 20
            \op s5, sp, 21
            \op s6, sp, 22
            \op s7, sp, 23
            \op s8, sp, 24
            \op s9, sp, 25
            \op s10, sp, 26
            \op s11, sp, 27
            \op t3, sp, 28
            \op t4, sp, 29
            \op t5, sp, 30
            \op t6, sp, 31
        .endm

        .macro PUSH_GENERAL_REGS
            PUSH_POP_GENERAL_REGS STR
        .endm
        .macro POP_GENERAL_REGS
            PUSH_POP_GENERAL_REGS LDR
        .endm

        .endif"
        );
    };
}
