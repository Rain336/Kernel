// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use proc_macro2::Span;
use std::fs;
use std::sync::OnceLock;
use syn::Error;
use toml::Table;

/// Path to the `Config.toml`
const CONFIG_TOML_PATH: &str = "Config.toml";

/// Statically cached and parsed `Config.toml` as a TOML [`Table`].
static CACHED_CONFIG: OnceLock<Table> = OnceLock::new();

/// Loads the `Config.toml` from the filesystem or returns a cached instance.
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
            format!("Could not parse Config.toml: {err}"),
        )
    })?;

    Ok(CACHED_CONFIG.get_or_init(|| table))
}

/// Only used by unit tests to set a `Config.toml` for testing.
#[cfg(test)]
pub fn set_config() {
    let _ = CACHED_CONFIG.set(toml::toml! {
        foo = 5
        bar.baz = true
        bar.string = "Hello World"
        bar.float = 3.1415926535

        [deeply.nested]
        table.value = false
        array = [1, 2, 3]
        table.array = [ { foo = "wow" }, { foo = 4 }, { bar = 69420 } ]

        [dragon]
        rawr = true
        size = 255
        likes = [ "pats", "hugs" ]

        [kobold]
        shiny = true
    });
}
