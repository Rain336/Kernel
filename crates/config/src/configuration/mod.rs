// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
mod all;
mod any;
mod not;
mod option;
mod resolver;
mod segment;

pub use all::*;
pub use any::*;
pub use not::*;
pub use option::*;
pub use segment::*;

use syn::parse::{Parse, ParseStream};
use syn::token;

/// Trait implemented by configuration predicates to allow evaluation.
pub trait EvaluatePredicate {
    /// Tries to evaluate this predicate to either match or not.
    fn evaluate(&self) -> syn::Result<bool>;
}

/// Represents a configuration predicate.
/// Either an option predicate (`foo = "bar"`) or a combinator predicate (`all(...)`, `any(...)` and `not(...)`).
pub enum ConfigurationPredicate {
    Option(ConfigurationOption),
    All(ConfigurationAll),
    Any(ConfigurationAny),
    Not(ConfigurationNot),
}

impl Parse for ConfigurationPredicate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let forked = input.fork();

        if let Ok(ident) = forked.parse::<syn::Ident>() {
            if forked.peek(token::Paren) {
                return if ident == "all" {
                    Ok(ConfigurationPredicate::All(input.parse()?))
                } else if ident == "any" {
                    Ok(ConfigurationPredicate::Any(input.parse()?))
                } else if ident == "not" {
                    Ok(ConfigurationPredicate::Not(input.parse()?))
                } else {
                    Err(input.error("Expected either 'all', 'any' or 'not'"))
                };
            }
        }

        Ok(ConfigurationPredicate::Option(input.parse()?))
    }
}

impl EvaluatePredicate for ConfigurationPredicate {
    fn evaluate(&self) -> syn::Result<bool> {
        match self {
            ConfigurationPredicate::Option(option) => option.evaluate(),
            ConfigurationPredicate::All(all) => all.evaluate(),
            ConfigurationPredicate::Any(any) => any.evaluate(),
            ConfigurationPredicate::Not(not) => not.evaluate(),
        }
    }
}
