use common::addr::{PhysAddr, VirtAddr};

#[export_name = "__internal_virtual_to_physical_kernel"]
fn virtual_to_physical_kernel(virt: VirtAddr) -> Option<PhysAddr> {
    crate::vmm::translate_address(virt)
}
