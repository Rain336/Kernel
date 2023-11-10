// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use proc_macro2::Span;
use std::fs;
use std::sync::OnceLock;
use syn::Error;
use toml::Table;

const CONFIG_TOML_PATH: &str = "Config.toml";

static CACHED_CONFIG: OnceLock<Table> = OnceLock::new();

pub fn get_config() -> syn::Result<&'static Table> {
    if let Some(table) = CACHED_CONFIG.get() {
        return Ok(table);
    }

    let file = fs::read_to_string(CONFIG_TOML_PATH).map_err(|err| {
        Error::new(
            Span::call_site(),
            format!("Could not read Config.toml: {err}"),
        )
    })?;
    let table = file.parse::<Table>().map_err(|err| {
        Error::new(
            Span::call_site(),
            format!("Couldn't parse Config.toml: {err}"),
        )
    })?;

    Ok(CACHED_CONFIG.get_or_init(|| table))
}

#[cfg(test)]
pub fn set_config(table: Table) {
    CACHED_CONFIG
        .set(table)
        .expect("Config already set by another test.");
}
