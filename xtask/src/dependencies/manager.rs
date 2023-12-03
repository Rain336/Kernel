// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use color_eyre::eyre::{bail, eyre};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use xshell::Shell;

use super::{Dependency, ResolvedDependency};

#[derive(Serialize, Deserialize)]
struct DependencyManifest {
    id: String,
    last_update: SystemTime,
    metadata: serde_json::Value,
}

const DEPS_MANIFEST_NAME: &str = "manifest.json";
const UPDATE_DELAY: Duration = Duration::from_secs(60 * 60 * 24);

pub struct DependencyManager {
    deps: PathBuf,
    manifests: Vec<DependencyManifest>,
}

impl DependencyManager {
    pub fn load(deps: PathBuf) -> Result<Self> {
        match fs::metadata(&deps) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => {
                return Err(eyre!(
                    "Deps directory ({}) is not a directory",
                    deps.display()
                ))
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {
                fs::create_dir(&deps)?;
            }
            Err(err) => bail!(err),
        }

        let manifests = match fs::read_to_string(deps.join(DEPS_MANIFEST_NAME)) {
            Ok(text) => serde_json::from_str(&text)?,
            Err(err) if err.kind() == ErrorKind::NotFound => Vec::new(),
            Err(err) => bail!(err),
        };

        Ok(DependencyManager { deps, manifests })
    }

    pub fn resolve(&mut self, dep: &impl Dependency, sh: &Shell) -> Result<ResolvedDependency> {
        let dir = sh.push_dir(&self.deps);

        match self.manifests.iter_mut().find(|x| x.id == dep.id()) {
            Some(manifest) => {
                if manifest.last_update.elapsed()? > UPDATE_DELAY {
                    dep.update(sh, &mut manifest.metadata)?;
                    manifest.last_update = SystemTime::now();
                    self.update_manifest()?;
                }
            }
            None => self.install_dependency(dep, sh)?,
        }

        drop(dir);

        Ok(ResolvedDependency {
            path: self.deps.join(dep.id()),
        })
    }

    fn install_dependency(&mut self, dep: &impl Dependency, sh: &Shell) -> Result<()> {
        let mut metadata = serde_json::Value::Null;

        dep.install(sh, &mut metadata)?;

        self.manifests.push(DependencyManifest {
            id: dep.id().to_string(),
            last_update: SystemTime::now(),
            metadata,
        });
        self.update_manifest()
    }

    fn update_manifest(&self) -> Result<()> {
        let json = serde_json::to_string(&self.manifests)?;
        let path = self.deps.join(DEPS_MANIFEST_NAME);
        fs::write(path, json)?;
        Ok(())
    }
}
