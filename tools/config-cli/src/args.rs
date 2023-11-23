// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! ## Program Arguments Module
//!
//! This module parses the program arguments into an enum [`ProgramInput`], which can then be run.
//!
use crate::cli::{self, Operation};
use crate::tui::{self, TuiModeInput};
use anyhow::{bail, Result};
use clap::{arg, value_parser, Arg, ArgAction, ArgGroup, Command, ValueHint};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Builds the clap command struct.
fn build_command() -> Command {
    Command::new("config")
        .about("command-line tool to edit Config.toml files")
        .long_about("A command-line tool to edit Config.toml files.\n\
        This tool provides edition in two modes, a cli mode using only command arguments to get or set multiple keys for a project.\n\
        And a terminal UI mode to interactively set options for a whole workspace or just a single project.")
        .disable_version_flag(true)
        .arg(
            arg!(-p --project <PATH> "Specifies the project or workspace folder to operate in")
            .value_parser(value_parser!(PathBuf))
            .value_hint(ValueHint::DirPath)
            .action(ArgAction::Append)
        )
        .arg(
            Arg::new("set")
            .short('s')
            .long("set")
            .action(ArgAction::Append)
            .num_args(2)
            .value_names(["KEY", "VALUE"])
            .help("Sets the given key to value, printing a value with format <KEY> = <VALUE>")
        )
        .arg(
            arg!(-g --get <KEY> "Gets the given key, printing a value with format <KEY> = <VALUE>")
            .action(ArgAction::Append)
        )
        .arg(
            arg!(-l --list [KEY] "Lists the sub keys of the given key or of the root key if no key is supplied")
            .action(ArgAction::Append)
        )
        .arg(arg!(-r --restrict <KEY> "Restricts the view of the TUI to the given key and it's sub-keys."))
        .group(
            ArgGroup::new("cli")
            .conflicts_with("tui")
            .multiple(true)
            .args(["set", "get", "list"])
        )
        .group(
            ArgGroup::new("tui")
            .conflicts_with("cli")
            .arg("restrict")
        )
}

/// Command-line input of the program.
pub enum ProgramInput {
    Cli(Vec<Operation>),
    Tui(TuiModeInput),
}

impl ProgramInput {
    /// Runs the program in the correct mode, based on it's arguments.
    pub fn run(self) -> Result<()> {
        match self {
            ProgramInput::Cli(operations) => cli::run(operations),
            ProgramInput::Tui(input) => tui::run(input),
        }
    }
}

/// Parses the program arguments and returns it as a [`ProgramInput`] enum.
pub fn parse() -> Result<ProgramInput> {
    let mut matches = build_command().get_matches();

    if matches.contains_id("cli") {
        let mut args = BTreeMap::new();

        if let Some(values) = matches.get_many::<String>("get") {
            let indices = matches.indices_of("get").expect("id has values");

            args.extend(indices.zip(values.cloned().map(Operation::Get)));
        }

        if let Some(values) = matches.get_many::<(String, String)>("set") {
            let indices = matches.indices_of("set").expect("id has values");

            args.extend(
                indices.zip(
                    values
                        .cloned()
                        .map(|(key, value)| Operation::Set(key, value)),
                ),
            );
        }

        if let Some(values) = matches.get_many::<Option<String>>("list") {
            let indices = matches.indices_of("list").expect("id has values");

            args.extend(indices.zip(values.cloned().map(Operation::List)));
        }

        if let Some(values) = matches.get_many::<PathBuf>("project") {
            let indices = matches.indices_of("project").expect("id has values");

            args.extend(indices.zip(values.cloned().map(Operation::Project)));
        }

        Ok(ProgramInput::Cli(args.into_values().collect()))
    } else {
        if let Some(projects) = matches.get_many::<PathBuf>("project") {
            if projects.len() > 1 {
                bail!("Only one project argument can be supplied in tui mode.");
            }
        }

        Ok(ProgramInput::Tui(TuiModeInput {
            project: matches.remove_one::<PathBuf>("project"),
            restrict: matches.remove_one::<String>("restrict"),
        }))
    }
}
