// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! ## CLI Mode
//!
//! In CLI mode the program processes a series of [`Operation`]s and prints a resulting line for each.
//!
mod get;
mod list;
mod set;
mod utils;

use anyhow::Result;
use std::fs;
#[cfg(test)]
use std::io::Write;
use std::path::PathBuf;
use toml_edit::Document;

/// Name of the Config.toml
const CONFIG_TOML_NAME: &str = "Config.toml";

/// An operation that can be executed in CLI mode.
pub enum Operation {
    /// A `Get` operation reads a value from the Config.toml
    Get(String),

    /// A `Set` operation sets a value in the Config.toml
    Set(String, String),

    /// A `List` operation lists the keys of a value in the Config.toml
    List(Option<String>),

    /// A `Project` operation changes the project and corresponding Config.toml that is operated on.
    Project(PathBuf),
}

/// Context storing state for evaluation of [`Operation`]s.
struct EvaluationContext {
    path: PathBuf,
    document: Option<Document>,
    dirty: bool,
}

impl EvaluationContext {
    /// Evaluates the given [`Operation`] in this context.
    fn evaluate(&mut self, op: Operation) {
        match op {
            Operation::Get(key) => {
                if let Some(document) = self.get_document() {
                    get::read_value(key, document.as_table_mut());
                }
            }
            Operation::Set(key, new_value) => {
                if let Some(document) = self.get_document() {
                    set::write_value(key, new_value, document.as_table_mut());
                }
            }
            Operation::List(key) => {
                if let Some(document) = self.get_document() {
                    list::list_value(key, document.as_table_mut(), ' ');
                }
            }
            Operation::Project(path) => self.switch_project(path),
        }
    }

    /// Gets the Config.toml of the current project as a [`Document`], if possible.
    /// If the Config.toml could not be loaded or parsed, an error is printed.
    fn get_document(&mut self) -> Option<&mut Document> {
        if self.document.is_none() {
            let text = match fs::read_to_string(&self.path) {
                Ok(text) => text,
                Err(err) => {
                    println!("ERROR: Failed to load {}: {err}", self.path.display());
                    return None;
                }
            };

            let document = match text.parse::<Document>() {
                Ok(document) => document,
                Err(err) => {
                    println!(
                        "ERROR: Failed to parse TOML for {}: {err}",
                        self.path.display()
                    );
                    return None;
                }
            };

            self.document = Some(document);
        }

        self.document.as_mut()
    }

    /// Switches the current project to the one at the given path.
    fn switch_project(&mut self, path: PathBuf) {
        if let Some(document) = &self.document {
            if self.dirty {
                if let Err(err) = fs::write(&self.path, document.to_string()) {
                    println!("ERROR: Failed to save {}: {err}", self.path.display());
                }
            }
        }

        self.path = path.join(CONFIG_TOML_NAME);
        self.document = None;
        self.dirty = false;
    }
}

impl Drop for EvaluationContext {
    fn drop(&mut self) {
        if let Some(document) = &self.document {
            if self.dirty {
                if let Err(err) = fs::write(&self.path, document.to_string()) {
                    println!("ERROR: Failed to save {}: {err}", self.path.display());
                }
            }
        }
    }
}

/// Runs the given vec of [`Operation`]s on a new context.
/// The context is initialized to the current directory as project.
pub fn run(operations: Vec<Operation>) -> Result<()> {
    let path = std::env::current_dir()?.join(CONFIG_TOML_NAME);

    let mut ctx = EvaluationContext {
        path,
        document: None,
        dirty: false,
    };

    for op in operations.into_iter() {
        ctx.evaluate(op);
    }

    Ok(())
}
