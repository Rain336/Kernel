// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

macro_rules! assert_parse {
    ($type:ty, $input:literal) => {
        if let Err(err) = syn::parse_str::<$type>($input) {
            assert!(false, "Expected parsing to succeed, but got: {err}");
        }
    };
}

macro_rules! assert_parse_fail {
    ($type:ty, $input:literal) => {
        if let Ok(result) = syn::parse_str::<$type>($input) {
            assert!(false, "Expected parsing to fail");
        }
    };
}