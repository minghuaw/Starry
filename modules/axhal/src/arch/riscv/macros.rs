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
    };
}
