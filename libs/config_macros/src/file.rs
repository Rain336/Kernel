use proc_macro::Span;
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
            Span::call_site().into(),
            format!("Could not read Config.toml: {err}"),
        )
    })?;
    let table = file.parse::<Table>().map_err(|err| {
        Error::new(
            Span::call_site().into(),
            format!("Couldn't parse Config.toml: {err}"),
        )
    })?;

    Ok(CACHED_CONFIG.get_or_init(|| table))
}
