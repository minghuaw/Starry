use super::frame::TrapFrame;
include_asm_marcos!();
core::arch::global_asm!(
    include_str!("stackful_trap.S"),
    trapframe_size = const core::mem::size_of::<TrapFrame>(),
);
