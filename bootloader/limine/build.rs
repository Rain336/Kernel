// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

fn main() {
    let script = std::env::current_dir().unwrap().join("linker.ld");

    println!("cargo:rustc-link-arg=-T{}", script.display());
    println!("cargo:rerun-if-changed=linker.ld");
}
