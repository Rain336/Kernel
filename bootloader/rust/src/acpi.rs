// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use bootloader_api::BootInfo;

pub fn get_rsdp_address(info: &BootInfo) -> u64 {
    info.rsdp_addr.into_option().unwrap_or_default()
}
