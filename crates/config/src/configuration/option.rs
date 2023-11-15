// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::{resolver, EvaluatePredicate, OptionSegment};
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Lit, Token};
use toml::Value;

/// Represents an option predicate (`foo = "bar"`).
pub struct ConfigurationOption {
    pub segments: Punctuated<OptionSegment, Token![.]>,
    pub equals: Option<Token![=]>,
    pub literal: Option<Lit>,
}

impl Parse for ConfigurationOption {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let segments = Punctuated::parse_separated_nonempty(input)?;
        let equals: Option<Token![=]> = input.parse()?;
        let literal = if equals.is_some() {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(ConfigurationOption {
            segments,
            equals,
            literal,
        })
    }
}

impl EvaluatePredicate for ConfigurationOption {
    fn evaluate(&self) -> syn::Result<bool> {
        let Some(value) = resolver::resolve(&self.segments)? else {
            return Ok(false);
        };

        if let Some(lit) = &self.literal {
            match (lit, value) {
                (Lit::Str(left), Value::String(right)) => Ok(left.value() == right),
                (Lit::Int(left), Value::Integer(right)) => Ok(left.base10_parse::<i64>()? == right),
                (Lit::Float(left), Value::Float(right)) => Ok(left.base10_parse::<f64>()? == right),
                (Lit::Bool(left), Value::Boolean(right)) => Ok(left.value == right),
                (Lit::Str(left), Value::Datetime(right)) => Ok(left.value() == right.to_string()),
                (_, value) => Err(syn::Error::new(
                    lit.span(),
                    format!(
                        "The provided literal of type '{}' doesn't match the TOML data type '{}'",
                        lit_type(lit),
                        value_type(&value)
                    ),
                )),
            }
        } else if let Value::Boolean(value) = value {
            Ok(value)
        } else {
            Err(syn::Error::new(
                Span::call_site(),
                "Key evaluates to a non-Boolean value",
            ))
        }
    }
}

/// Returns a type string for a [`Lit`].
fn lit_type(lit: &Lit) -> &'static str {
    match lit {
        Lit::Str(_) => "string",
        Lit::ByteStr(_) => "byte string",
        Lit::Byte(_) => "byte",
        Lit::Char(_) => "char",
        Lit::Int(_) => "integer",
        Lit::Float(_) => "float",
        Lit::Bool(_) => "boolean",
        _ => "unknown",
    }
}

/// Returns a type string for a [`Value`].
fn value_type(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "string",
        Value::Integer(_) => "integer",
        Value::Float(_) => "float",
        Value::Boolean(_) => "boolean",
        Value::Datetime(_) => "date time",
        Value::Array(_) => "array",
        Value::Table(_) => "table",
    }
}

#[cfg(test)]
mod test {
    use crate::configuration::ConfigurationOption;
    use crate::macros::*;

    #[test]
    fn test_parse() {
        assert_parse!(ConfigurationOption, "foo");
        assert_parse!(ConfigurationOption, "foo = \"bar\"");
        assert_parse!(ConfigurationOption, "foo = 123");
        assert_parse!(ConfigurationOption, "foo = true");
        assert_parse!(ConfigurationOption, "foo.bar = true");
        assert_parse!(ConfigurationOption, "dragon.\"hoard\".kobold");
        assert_parse!(ConfigurationOption, "dragon.\"hoard\".rawr = 255");

        assert_parse_fail!(ConfigurationOption, "kobold =");
        assert_parse_fail!(ConfigurationOption, "kobold.\"shiny");
        assert_parse_fail!(ConfigurationOption, "kobold.\"shiny =");
    }
}
