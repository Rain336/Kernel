// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::utils::CommandContext;
use color_eyre::Result;
use ignore::overrides::{Override, OverrideBuilder};
use ignore::types::TypesBuilder;
use ignore::{WalkBuilder, WalkState};
use std::fs;
use std::path::Path;

const LICENSE_DETECT_STRING: &str =
    "// This Source Code Form is subject to the terms of the Mozilla Public\n";
const LICENSE_TEXT: &str = r"// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
";

pub fn run(ctx: CommandContext) -> Result<()> {
    let types = TypesBuilder::new().add_defaults().select("rust").build()?;
    let walk = WalkBuilder::new("./")
        .overrides(create_override(ctx.workspace_directory())?)
        .types(types)
        .build_parallel();

    walk.run(|| {
        Box::new(|entry| {
            let Ok(entry) = entry else {
                return WalkState::Skip;
            };

            let Some(ty) = entry.file_type() else {
                return WalkState::Continue;
            };

            if !ty.is_file() {
                return WalkState::Continue;
            }

            let mut content = match fs::read_to_string(entry.path()) {
                Ok(content) => content,
                Err(err) => {
                    println!("Could not read {}: {err}", entry.path().display());
                    return WalkState::Continue;
                }
            };

            if !content.starts_with(LICENSE_DETECT_STRING) {
                content.insert_str(0, LICENSE_TEXT);

                match fs::write(entry.path(), content) {
                    Ok(()) => println!("Updated: {}", entry.path().display()),
                    Err(err) => {
                        println!("Could not update {}: {err}", entry.path().display());
                        return WalkState::Continue;
                    }
                }
            }

            WalkState::Continue
        })
    });

    Ok(())
}

fn create_override(path: &Path) -> Result<Override> {
    Ok(OverrideBuilder::new(path)
        .add("!libs/")?
        .add("!target/")?
        .build()?)
}
