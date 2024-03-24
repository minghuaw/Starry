use core::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use riscv::register::sstatus::{self, Sstatus};

use crate::ContextArgs;

/// Saved registers when a trap (interrupt or exception) occurs.
#[repr(C)]
#[derive(Clone, Copy)]
// 上下文
pub struct TrapFrame {
    /// All general registers including x0
    pub x: [usize; 32],
    /// Supervisor Status Register.
    pub sstatus: Sstatus,
    /// Supervisor Exception Program Counter.
    pub sepc: usize,
    /// 浮点数寄存器
    pub fsx: [usize; 2],
}

impl Debug for TrapFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TrapFrame")
            .field("ra", &self.x[1])
            .field("sp", &self.x[2])
            .field("gp", &self.x[3])
            .field("tp", &self.x[4])
            .field("t0", &self.x[5])
            .field("t1", &self.x[6])
            .field("t2", &self.x[7])
            .field("s0", &self.x[8])
            .field("s1", &self.x[9])
            .field("a0", &self.x[10])
            .field("a1", &self.x[11])
            .field("a2", &self.x[12])
            .field("a3", &self.x[13])
            .field("a4", &self.x[14])
            .field("a5", &self.x[15])
            .field("a6", &self.x[16])
            .field("a7", &self.x[17])
            .field("s2", &self.x[18])
            .field("s3", &self.x[19])
            .field("s4", &self.x[20])
            .field("s5", &self.x[21])
            .field("s6", &self.x[22])
            .field("s7", &self.x[23])
            .field("s8", &self.x[24])
            .field("s9", &self.x[25])
            .field("s10", &self.x[26])
            .field("s11", &self.x[27])
            .field("t3", &self.x[28])
            .field("t4", &self.x[29])
            .field("t5", &self.x[30])
            .field("t6", &self.x[31])
            .field("sstatus", &self.sstatus)
            .field("sepc", &self.sepc)
            .field("fsx", &self.fsx)
            .finish()
    }
}

impl TrapFrame {
    // 创建上下文信息
    #[inline]
    pub fn new() -> Self {
        TrapFrame {
            x: [0usize; 32],
            sstatus: sstatus::read(),
            sepc: 0,
            fsx: [0; 2],
        }
    }

    /// 用于第一次进入应用程序时的初始化
    pub fn app_init_context(app_entry: usize, user_sp: usize) -> Self {
        // 当前版本的riscv不支持使用set_spp函数，需要手动修改
        // 修改当前的sstatus为User，即是第8位置0
        let mut trap_frame = TrapFrame::new();
        trap_frame[ContextArgs::SP] = user_sp;
        // info!("app_entry: {:#x}", app_entry);
        trap_frame[ContextArgs::SEPC] = app_entry;
        trap_frame.sstatus.set_spp(sstatus::SPP::User);
        // When modifying the CSR, the SIE bit must be cleared
        trap_frame.sstatus.set_sie(false);
        // trap_frame.sstatus =
        //     unsafe { (*(&sstatus as *const Sstatus as *const usize) & !(1 << 8)) & !(1 << 1) };
        unsafe {
            // a0为参数个数
            // a1存储的是用户栈底，即argv
            trap_frame[ContextArgs::ARG0] = *(user_sp as *const usize);
            trap_frame[ContextArgs::ARG1] = *(user_sp as *const usize).add(1);
        }
        trap_frame
    }
}

impl TrapFrame {
    #[inline]
    pub fn args(&self) -> [usize; 6] {
        self.x[10..16].try_into().expect("args slice force convert")
    }

    #[inline]
    pub fn syscall_ok(&mut self) {
        self.sepc += 4;
    }
}

impl Index<ContextArgs> for TrapFrame {
    type Output = usize;

    fn index(&self, index: ContextArgs) -> &Self::Output {
        match index {
            ContextArgs::SEPC => &self.sepc,
            ContextArgs::RA => &self.x[1],
            ContextArgs::SP => &self.x[2],
            ContextArgs::RET => &self.x[10],
            ContextArgs::ARG0 => &self.x[10],
            ContextArgs::ARG1 => &self.x[11],
            ContextArgs::ARG2 => &self.x[12],
            ContextArgs::TLS => &self.x[4],
            ContextArgs::SYSCALL => &self.x[17],
        }
    }
}

impl IndexMut<ContextArgs> for TrapFrame {
    fn index_mut(&mut self, index: ContextArgs) -> &mut Self::Output {
        match index {
            ContextArgs::SEPC => &mut self.sepc,
            ContextArgs::RA => &mut self.x[1],
            ContextArgs::SP => &mut self.x[2],
            ContextArgs::RET => &mut self.x[10],
            ContextArgs::ARG0 => &mut self.x[10],
            ContextArgs::ARG1 => &mut self.x[11],
            ContextArgs::ARG2 => &mut self.x[12],
            ContextArgs::TLS => &mut self.x[4],
            ContextArgs::SYSCALL => &mut self.x[17],
        }
    }
}
