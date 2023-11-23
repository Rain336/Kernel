// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! # Compile-time Configuration Framework
//!
//! This proc-macro library implements microdragon's compile-time configuration framework
//! The frameworks reads in a `Config.toml` file in the same directory as your project's `Cargo.toml`
//! The framework consists of three macros
//!
//! - [`value`] Resolves to a value from the `Config.toml`
//! - [`config`] works like [`core::cfg`] but using values form the `Config.toml`
//! - [`config_attr`] works like the `cfg_attr` built-in macro but using values form the `Config.toml`
use attr::ConfigAttrMacroInput;
use configuration::{ConfigurationPredicate, EvaluatePredicate};
use proc_macro::TokenStream;
use syn::parse_macro_input;
use value::ValueMacroInput;

#[macro_use]
mod macros;
mod attr;
mod configuration;
mod file;
mod value;

/// Inserts values form the `Config.toml` using a string key as the first parameter.
/// Allows supplying a default value as a second parameter that is inserted if the key is missing in the `Config.toml`.
///
/// ```ignore
/// value!("foo.bar")
/// ```
///
/// ```ignore
/// value!("foo.baz", 15)
/// ```
#[proc_macro]
pub fn value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ValueMacroInput);

    input.run().into()
}

/// Can be attached to an item to only include it if the predicate, provided as a parameter, returns true.
/// This macros supports all combinators that [`core::cfg`] supports (`all(...)`, `any(...)` and `not(...)`).
/// In addition to comparing with strings, this also supports comparing with integers, floats and boolean literals.
/// Since TOML is a typed format, comparing with the wrong types is an error.
///
/// ```ignore
/// #[config(foo.bar)]
/// compile_error!("foo.bar is not set or false");
/// ```
///
/// ```ignore
/// #[config(dragon.rawr = 255)]
/// compile_error!("dragon.rawr is 255, way too much");
/// ```
///
/// ```ignore
/// #[config(all(dragon.rawr = 255, dragon.size = "big", not(dragon.micro)))]
/// fn allow_rawr() -> bool { true }
/// ```
#[proc_macro_attribute]
pub fn config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let predicate = parse_macro_input!(attr as ConfigurationPredicate);

    let result = match predicate.evaluate() {
        Ok(result) => result,
        Err(err) => return err.to_compile_error().into(),
    };

    if result {
        item
    } else {
        TokenStream::new()
    }
}

/// Can be attached to an item to conditionally add a set of attributes to it.
/// The first parameter is the predicate to check if the attribute should be added.
/// The following parameters are the attributes that should be added.
/// This macros supports all combinators that [`core::cfg`] supports (`all(...)`, `any(...)` and `not(...)`).
/// In addition to comparing with strings, this also supports comparing with integers, floats and boolean literals.
/// Since TOML is a typed format, comparing with the wrong types is an error.
#[proc_macro_attribute]
pub fn config_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as ConfigAttrMacroInput);

    input.run(item.into()).into()
}
