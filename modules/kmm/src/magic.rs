// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use common::addr::{PhysAddr, VirtAddr};

#[export_name = "__internal_virtual_to_physical_kernel"]
fn virtual_to_physical_kernel(virt: VirtAddr) -> Option<PhysAddr> {
    crate::vmm::translate_address(virt)
}
