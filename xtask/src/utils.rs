// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::dependencies::{Dependency, DependencyManager, ResolvedDependency};
use crate::fs::FileSystem;
use color_eyre::Result;
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};

const DEPS_DIRECTORY_NAME: &str = "deps";

pub struct CommandContext {
    shell: Shell,
    deps: DependencyManager,
    fs: FileSystem,
    workspace: PathBuf,
    target: PathBuf,
}

impl CommandContext {
    pub fn new() -> Result<Self> {
        let shell = Shell::new()?;
        let workspace = get_workspace_dir(&shell)?;
        let deps = DependencyManager::load(workspace.join(DEPS_DIRECTORY_NAME))?;
        let target = workspace.join("target");

        Ok(CommandContext {
            shell,
            deps,
            fs: FileSystem::new(),
            workspace,
            target,
        })
    }

    pub fn shell(&self) -> &Shell {
        &self.shell
    }

    pub fn file_system(&self) -> &FileSystem {
        &self.fs
    }

    pub fn file_system_mut(&mut self) -> &mut FileSystem {
        &mut self.fs
    }

    pub fn workspace_directory(&self) -> &Path {
        &self.workspace
    }

    pub fn target_directory(&self) -> &Path {
        &self.target
    }

    pub fn resolve_dependency(&mut self, dep: &impl Dependency) -> Result<ResolvedDependency> {
        self.deps.resolve(dep, &self.shell)
    }
}

fn get_workspace_dir(sh: &Shell) -> Result<PathBuf> {
    let path = cmd!(
        sh,
        "cargo locate-project --workspace --message-format=plain"
    )
    .quiet()
    .read()?;
    let toml = PathBuf::from(path);

    Ok(toml.parent().unwrap().into())
}
