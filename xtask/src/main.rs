// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
mod arguments;
mod build;
mod dependencies;
mod fs;
mod iso;
mod license;
mod run;
mod utils;

use arguments::ProgramArguments;
use clap::Parser;
use color_eyre::Result;
use utils::CommandContext;

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = ProgramArguments::parse();
    let ctx = CommandContext::new()?;

    match args {
        ProgramArguments::Build(build) => build.run(),
        ProgramArguments::Run(run) => run.run(ctx),
        ProgramArguments::Iso(iso) => iso.run(ctx),
        ProgramArguments::License => license::run(ctx),
    }
}
