// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use limine::RsdpRequest;

static RSDP_REQUEST: RsdpRequest = RsdpRequest::new(0);

pub fn get_rsdp_address() -> u64 {
    if let Some(response) = RSDP_REQUEST.get_response().get() {
        if let Some(rsdp) = response.address.as_ptr() {
            return rsdp as u64;
        }
    }

    0
}
