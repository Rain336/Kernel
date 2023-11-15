// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::{ConfigurationPredicate, EvaluatePredicate};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, Token};

/// Represents the `all(...)` combinator predicate.
pub struct ConfigurationAll {
    pub ident: syn::Ident,
    pub paren_token: token::Paren,
    pub inner: Punctuated<ConfigurationPredicate, Token![,]>,
}

impl Parse for ConfigurationAll {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident != "all" {
            return Err(syn::Error::new(ident.span(), "Expected 'all'"));
        }

        let content;

        Ok(ConfigurationAll {
            ident,
            paren_token: parenthesized!(content in input),
            inner: Punctuated::parse_terminated(&content)?,
        })
    }
}

impl EvaluatePredicate for ConfigurationAll {
    fn evaluate(&self) -> syn::Result<bool> {
        if self.inner.is_empty() {
            return Ok(false);
        }

        for predicate in &self.inner {
            if !predicate.evaluate()? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use crate::configuration::ConfigurationAll;
    use crate::macros::*;

    #[test]
    fn test_parse() {
        assert_parse!(ConfigurationAll, "all()");
        assert_parse!(ConfigurationAll, "all(foo)");
        assert_parse!(ConfigurationAll, "all(foo = \"bar\")");
        assert_parse!(ConfigurationAll, "all(foo, not(bar))");
        assert_parse!(ConfigurationAll, "all(foo = \"bar\", not(bar))");
        assert_parse!(ConfigurationAll, "all(any(foo))");

        assert_parse_fail!(ConfigurationAll, "all(");
        assert_parse_fail!(ConfigurationAll, "any()");
    }
}
