// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::configuration::{ConfigurationPredicate, EvaluatePredicate};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Item, Meta, Token};

/// Struct defining the parameters to the [`crate::config_attr`] macro.
pub struct ConfigAttrMacroInput {
    predicate: ConfigurationPredicate,
    _comma: Token![,],
    attrs: Punctuated<Meta, Token![,]>,
}

impl ConfigAttrMacroInput {
    /// Runs the proc-macro logic on the given item.
    pub fn run(self, item: TokenStream) -> TokenStream {
        let result = match self.predicate.evaluate() {
            Ok(result) => result,
            Err(err) => return err.to_compile_error(),
        };

        if result {
            let mut item: Item = match syn::parse2(item) {
                Ok(item) => item,
                Err(err) => return err.to_compile_error(),
            };

            for meta in self.attrs {
                let attr = Attribute {
                    pound_token: Default::default(),
                    style: syn::AttrStyle::Outer,
                    bracket_token: Default::default(),
                    meta,
                };

                if let Err(err) = push_attribute_into_item(attr, &mut item) {
                    return err.to_compile_error();
                }
            }

            item.into_token_stream()
        } else {
            item
        }
    }
}

impl Parse for ConfigAttrMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ConfigAttrMacroInput {
            predicate: input.parse()?,
            _comma: input.parse()?,
            attrs: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

/// Pushes the given `attr` as an attribute into the given `item`.
/// Returns an error if the given item is `Verbatim` or not supported.
fn push_attribute_into_item(attr: Attribute, item: &mut Item) -> syn::Result<()> {
    match item {
        Item::Const(i) => i.attrs.push(attr),
        Item::Enum(i) => i.attrs.push(attr),
        Item::ExternCrate(i) => i.attrs.push(attr),
        Item::Fn(i) => i.attrs.push(attr),
        Item::ForeignMod(i) => i.attrs.push(attr),
        Item::Impl(i) => i.attrs.push(attr),
        Item::Macro(i) => i.attrs.push(attr),
        Item::Mod(i) => i.attrs.push(attr),
        Item::Static(i) => i.attrs.push(attr),
        Item::Struct(i) => i.attrs.push(attr),
        Item::Trait(i) => i.attrs.push(attr),
        Item::TraitAlias(i) => i.attrs.push(attr),
        Item::Type(i) => i.attrs.push(attr),
        Item::Union(i) => i.attrs.push(attr),
        Item::Use(i) => i.attrs.push(attr),
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "#[config_attr(..., ...)] cannot be applied to this item.",
            ))
        }
    }

    Ok(())
}
