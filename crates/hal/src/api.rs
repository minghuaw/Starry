use crate_interface::call_interface;

#[allow(unused_imports)]
use fdt::node::FdtNode;

use crate::{TrapFrame, TrapType};

#[crate_interface::def_interface]
pub trait ArchInterface {
    /// kernel interrupt
    fn kernel_interrupt(ctx: &mut TrapFrame, from_user: bool, trap_type: TrapType);
    /// init log
    fn init_logging();
    /// add a memory region from start to end
    fn add_memory_region(start: usize, end: usize);
    /// init the allocator
    fn init_allocator();
    /// kernel main function, entry point.
    fn main(hartid: usize);
    // /// Alloc a persistent memory page.
    // fn frame_alloc_persist() -> PhysPage {
    //     unimplemented!()
    // }
    // /// Unalloc a persistent memory page
    // fn frame_unalloc(_ppn: PhysPage) {
    //     unimplemented!()
    // }
    /// Preprare drivers.
    fn prepare_drivers();
    /// Try to add device through FdtNode
    fn try_to_add_device(fdt_node: &FdtNode);
}

/// Kernel main function, entry point.
pub(crate) fn kernel_main(hartid: usize) {
    call_interface!(ArchInterface::main, hartid);
}

/// Kernel interrupt handler.
pub fn kernel_interrupt(ctx: &mut TrapFrame, from_user: bool, trap_type: TrapType) {
    call_interface!(ArchInterface::kernel_interrupt, ctx, from_user, trap_type);
}

/// Init logging.
pub fn init_logging() {
    call_interface!(ArchInterface::init_logging);
}

#[allow(dead_code)]
/// Add a memory region.
pub(crate) fn add_memory_region(start: usize, end: usize) {
    call_interface!(ArchInterface::add_memory_region, start, end);
}

#[allow(dead_code)]
/// Init the allocator.
pub(crate) fn init_allocator() {
    call_interface!(ArchInterface::init_allocator);
}

// #[allow(dead_code)]
// /// Alloc a persistent memory page.
// pub(crate) fn frame_alloc_persist() -> PhysPage {
//     call_interface!(ArchInterface::frame_alloc_persist)
// }

// #[allow(dead_code)]
// /// Unalloc a persistent memory page.
// pub(crate) fn frame_unalloc(ppn: PhysPage) {
//     call_interface!(ArchInterface::frame_unalloc, ppn);
// }

#[allow(dead_code)]
/// Prepare drivers.
pub(crate) fn prepare_drivers() {
    call_interface!(ArchInterface::prepare_drivers);
}

#[allow(dead_code)]
/// Try to add device through FdtNode.
pub(crate) fn try_to_add_device(fdt_node: &FdtNode) {
    call_interface!(ArchInterface::try_to_add_device, fdt_node);
}
