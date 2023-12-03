// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::download::DownloadDependency;
use super::hooks::{build_limine, extract_omvf};
use super::GitDependency;
use super::rust::RustBootloaderDependency;

pub static LIMINE_DEPENDENCY: GitDependency = GitDependency {
    id: "limine",
    repo_url: "https://github.com/limine-bootloader/limine",
    branch: Some("v5.x-branch-binary"),
    post_install: Some(build_limine),
};

pub static OVMF_DEPENDENCY: DownloadDependency = DownloadDependency {
    id: "OVMF",
    url: "https://github.com/rust-osdev/ovmf-prebuilt/releases/download/edk2-stable202211-r1/edk2-stable202211-r1-bin.tar.xz",
    file_name: "edk2-stable202211-r1-bin.tar.xz",
    post_install: Some(extract_omvf),
};

pub static RUST_BOOTLOADER: RustBootloaderDependency = RustBootloaderDependency {
    version: "0.11.4"
};
