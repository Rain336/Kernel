// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::Lit;

/// Represents a segment in a key.
/// Con either be an identifier (`foo`), a string (`"foo bar"`) or an int (`0`).
pub enum OptionSegment {
    Ident(syn::Ident),
    String(syn::LitStr),
    Int(syn::LitInt),
}

impl OptionSegment {
    pub fn span(&self) -> Span {
        match self {
            OptionSegment::Ident(ident) => ident.span(),
            OptionSegment::String(str) => str.span(),
            OptionSegment::Int(int) => int.span(),
        }
    }
}

impl Parse for OptionSegment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.step(|cursor| {
            if let Some((lit, rest)) = cursor.literal() {
                match Lit::new(lit) {
                    Lit::Str(lit) => return Ok((OptionSegment::String(lit), rest)),
                    Lit::Int(lit) => return Ok((OptionSegment::Int(lit), rest)),
                    _ => {}
                }
            }

            if let Some((ident, rest)) = cursor.ident() {
                return Ok((OptionSegment::Ident(ident), rest));
            }

            Err(cursor.error("Expected string literal or identifier"))
        })
    }
}

#[cfg(test)]
mod test {
    use crate::configuration::OptionSegment;
    use crate::macros::*;

    #[test]
    fn test_parse() {
        assert_parse!(OptionSegment, "kobold");
        assert_parse!(OptionSegment, "\"kobold\"");

        assert_parse_fail!(OptionSegment, "\"kobold");
        assert_parse_fail!(OptionSegment, "kobold\"");
    }
}
