use syn::parse::{Parse, ParseStream};
use syn::{Expr, LitStr, Token};

pub struct ConfigQuery {
    pub option: LitStr,
    pub comma: Option<Token![,]>,
    pub default: Option<Expr>,
}

impl Parse for ConfigQuery {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let option = input.parse()?;
        let (comma, default) = if let Ok(comma) = input.parse() {
            (Some(comma), input.parse().ok())
        } else {
            (None, None)
        };

        Ok(ConfigQuery {
            option,
            comma,
            default,
        })
    }
}
