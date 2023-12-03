// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use color_eyre::Result;
use std::fs;
use std::path::Path;
use xshell::{cmd, Shell};

pub fn build_limine(sh: &Shell) -> Result<()> {
    if cfg!(windows) {
        return Ok(());
    }

    if sh.path_exists("limine") {
        sh.remove_path("limine")?;
    }

    cmd!(sh, "make").run()?;

    Ok(())
}

pub fn extract_omvf(tar: &Path, sh: &Shell) -> Result<()> {
    println!("Extracting OVMF...");
    cmd!(sh, "tar xvf {tar}").run()?;
    sh.remove_path(tar)?;

    let ovmf = sh.current_dir().join("OVMF");
    if ovmf.exists() {
        fs::remove_dir_all(&ovmf)?;
    }
    fs::rename(sh.current_dir().join("edk2-stable202211-r1-bin"), ovmf)?;

    Ok(())
}
