// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use bitflags::bitflags;

bitflags! {
    pub struct ErrorStatus : u32 {
        const REDIRECTABLE_IPI = 1 << 4;
        const SEND_ILLEGAL_VECTOR = 1 << 5;
        const RECEIVE_ILLEGAL_VECTOR = 1 << 6;
        const ILLEGAL_REGISTER_ADDRESS = 1 << 7;
    }
}
