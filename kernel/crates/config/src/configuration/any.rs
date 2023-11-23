// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::{ConfigurationPredicate, EvaluatePredicate};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, Token};

/// Represents the `any(...)` combinator predicate.
pub struct ConfigurationAny {
    pub ident: syn::Ident,
    pub paren_token: token::Paren,
    pub inner: Punctuated<ConfigurationPredicate, Token![,]>,
}

impl Parse for ConfigurationAny {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident != "any" {
            return Err(syn::Error::new(ident.span(), "Expected 'any'"));
        }

        let content;

        Ok(ConfigurationAny {
            ident,
            paren_token: parenthesized!(content in input),
            inner: Punctuated::parse_terminated(&content)?,
        })
    }
}

impl EvaluatePredicate for ConfigurationAny {
    fn evaluate(&self) -> syn::Result<bool> {
        for predicate in &self.inner {
            if predicate.evaluate()? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use crate::configuration::ConfigurationAny;
    use crate::macros::*;

    #[test]
    fn test_parse() {
        assert_parse!(ConfigurationAny, "any()");
        assert_parse!(ConfigurationAny, "any(foo)");
        assert_parse!(ConfigurationAny, "any(foo = \"bar\")");
        assert_parse!(ConfigurationAny, "any(foo, not(bar))");
        assert_parse!(ConfigurationAny, "any(foo = \"bar\", not(bar))");
        assert_parse!(ConfigurationAny, "any(all(foo))");

        assert_parse_fail!(ConfigurationAny, "any(");
        assert_parse_fail!(ConfigurationAny, "all()");
    }
}
