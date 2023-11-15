// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::OptionSegment;
use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::Token;
use toml::value::Array;
use toml::{Table, Value};

/// Helper enum to transparently traverse an array or table.
#[derive(Clone, Copy)]
enum TableOrArray<'a> {
    Table(&'a Table),
    Array(&'a Array),
}

impl<'a> TableOrArray<'a> {
    /// Gets the value corresponding to the given segment string from the inner table or array.
    fn get(self, segment: &OptionSegment) -> syn::Result<Option<&'a Value>> {
        Ok(match self {
            TableOrArray::Table(x) => match segment {
                OptionSegment::Ident(ident) => x.get(&ident.to_string()),
                OptionSegment::String(str) => x.get(&str.value()),
                OptionSegment::Int(int) => x.get(int.base10_digits()),
            },
            TableOrArray::Array(x) => match segment {
                OptionSegment::Int(int) => {
                    let index = int.base10_parse::<usize>().map_err(|err| syn::Error::new(
                        segment.span(),
                        format!("Could not convert key segment '{}' into an array index, got error: {}", int.base10_digits(), err),
                    ))?;
                    x.get(index)
                }
                _ => {
                    return Err(syn::Error::new(
                        segment.span(),
                        "Expected an array index at this key segment",
                    ));
                }
            },
        })
    }
}

/// Tries to resolve the given list of [`OptionSegment`]s to a TOML [`Value`].
pub fn resolve(punctuated: &Punctuated<OptionSegment, Token![.]>) -> syn::Result<Option<Value>> {
    let table = crate::file::get_config()?;
    let last = punctuated.len() - 1;

    let mut current = TableOrArray::Table(table);
    for (idx, segment) in punctuated.iter().enumerate() {
        match current.get(segment)? {
            Some(value) => match value {
                Value::Array(x) => current = TableOrArray::Array(x),
                Value::Table(x) => current = TableOrArray::Table(x),
                _ if idx == last => return Ok(Some(value.clone())),
                _ => {
                    return Err(syn::Error::new(
                        segment.span(),
                        "Key segment is a non-Table / non-Array value",
                    ))
                }
            },
            None => return Ok(None),
        }
    }

    Err(syn::Error::new(
        Span::call_site(),
        "Key resolves to a table or Array",
    ))
}
