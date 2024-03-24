use core::{
    fmt::{Debug, Display},
    mem::size_of,
    ops::Add,
};

use memory_addr::{PhysAddr, VirtAddr, PAGE_SIZE_4K};

use crate::VIRT_ADDR_START;

impl From<PhysPage> for PhysAddr {
    fn from(value: PhysPage) -> Self {
        // Self(value.0 << 12)
        PhysAddr::from(value.0 << 12)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPage(pub(crate) usize);

impl From<usize> for PhysPage {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<PhysAddr> for PhysPage {
    fn from(value: PhysAddr) -> Self {
        Self(value.as_usize() >> 12)
    }
}

impl From<PhysPage> for usize {
    fn from(value: PhysPage) -> Self {
        value.0
    }
}

impl Add<PhysPage> for PhysPage {
    type Output = PhysPage;

    fn add(self, rhs: PhysPage) -> Self::Output {
        PhysPage(self.0 + rhs.0)
    }
}

impl Add<usize> for PhysPage {
    type Output = PhysPage;

    fn add(self, rhs: usize) -> Self::Output {
        PhysPage(self.0 + rhs)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPage(pub(crate) usize);
impl From<VirtAddr> for VirtPage {
    fn from(value: VirtAddr) -> Self {
        Self(value.as_usize() >> 12)
    }
}
impl From<usize> for VirtPage {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl PhysPage {
    #[inline]
    pub const fn new(ppn: usize) -> Self {
        Self(ppn)
    }

    #[inline]
    pub const fn from_addr(addr: usize) -> Self {
        Self(addr >> 12)
    }

    #[inline]
    pub const fn to_addr(&self) -> usize {
        self.0 << 12
    }

    #[inline]
    pub const fn get_buffer(&self) -> &'static mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (self.0 << 12 | VIRT_ADDR_START) as *mut u8,
                PAGE_SIZE_4K,
            )
        }
    }

    #[inline]
    pub fn copy_value_from_another(&self, ppn: PhysPage) {
        self.get_buffer().copy_from_slice(&ppn.get_buffer());
        #[cfg(c906)]
        unsafe {
            asm!(".long 0x0010000b"); // dcache.all
            asm!(".long 0x01b0000b"); // sync.is
        }
    }

    #[inline]
    pub fn drop_clear(&self) {
        // self.get_buffer().fill(0);
        unsafe {
            core::slice::from_raw_parts_mut(
                (self.0 << 12 | VIRT_ADDR_START) as *mut usize,
                PAGE_SIZE_4K / size_of::<usize>(),
            )
            .fill(0);
        }
        #[cfg(c906)]
        unsafe {
            asm!(".long 0x0010000b"); // dcache.all
            asm!(".long 0x01b0000b"); // sync.is
        }
    }
}

impl Add<usize> for VirtPage {
    type Output = VirtPage;

    fn add(self, rhs: usize) -> Self::Output {
        VirtPage(self.0 + rhs)
    }
}

impl From<VirtPage> for VirtAddr {
    fn from(value: VirtPage) -> Self {
        // Self(value.to_addr())
        VirtAddr::from(value.to_addr())
    }
}

impl VirtPage {
    #[inline]
    pub const fn new(vpn: usize) -> Self {
        Self(vpn)
    }

    #[inline]
    pub const fn from_addr(addr: usize) -> Self {
        Self(addr >> 12)
    }
    #[inline]
    pub const fn to_addr(&self) -> usize {
        self.0 << 12
    }
}

impl Display for PhysPage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Display for VirtPage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Debug for PhysPage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Debug for VirtPage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}
