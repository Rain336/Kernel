// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::{ConfigurationPredicate, EvaluatePredicate};
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, token};

/// Represents the `not(...)` combinator predicate.
pub struct ConfigurationNot {
    pub ident: syn::Ident,
    pub paren_token: token::Paren,
    pub inner: Box<ConfigurationPredicate>,
}

impl Parse for ConfigurationNot {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident != "not" {
            return Err(syn::Error::new(ident.span(), "Expected 'not'"));
        }

        let content;

        Ok(ConfigurationNot {
            ident,
            paren_token: parenthesized!(content in input),
            inner: content.parse()?,
        })
    }
}

impl EvaluatePredicate for ConfigurationNot {
    fn evaluate(&self) -> syn::Result<bool> {
        self.inner.evaluate().map(|result| !result)
    }
}

#[cfg(test)]
mod test {
    use crate::configuration::ConfigurationNot;
    use crate::macros::*;

    #[test]
    fn test_parse() {
        assert_parse!(ConfigurationNot, "not(foo)");
        assert_parse!(ConfigurationNot, "not(foo = \"bar\")");
        assert_parse!(ConfigurationNot, "not(any(foo))");

        assert_parse_fail!(ConfigurationNot, "not(");
        assert_parse_fail!(ConfigurationNot, "not()");
        assert_parse_fail!(ConfigurationNot, "any()");
    }
}
