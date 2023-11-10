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

#[cfg(test)]
mod test {
    use super::ConfigQuery;

    macro_rules! assert_query {
        ($input:expr, $option:expr, $comma:pat, $default:pat) => {
            let stream = syn::parse_str::<ConfigQuery>($input).unwrap();
            assert_eq!(stream.option.value(), $option);
            assert!(matches!(stream.comma, $comma));
            assert!(matches!(stream.default, $default));
        };
        ($input:expr, $option:expr, $comma:pat) => {
            let stream = syn::parse_str::<ConfigQuery>($input).unwrap();
            assert_eq!(stream.option.value(), $option);
            assert!(matches!(stream.comma, $comma));
            assert!(matches!(stream.default, None));
        };
        ($input:expr, $option:expr) => {
            let stream = syn::parse_str::<ConfigQuery>($input).unwrap();
            assert_eq!(stream.option.value(), $option);
            assert!(matches!(stream.comma, None));
            assert!(matches!(stream.default, None));
        };
    }

    #[test]
    fn test_parse() {
        assert_query!("\"foo\"", "foo");

        assert_query!("\"foo.bar\"", "foo.bar");

        assert_query!("\"foo.'bar'\"", "foo.'bar'");

        assert_query!("\"foo.\\\"bar\\\"\"", "foo.\"bar\"");

        assert_query!("\"foo.bar.baz\",", "foo.bar.baz", Some(_));

        assert_query!("\"foo.'bar'\", true", "foo.'bar'", Some(_), Some(_));
    }
}
