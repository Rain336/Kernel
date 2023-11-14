// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::{file, ConfigQuery};
use proc_macro2::Span;
use syn::{Error, Expr, ExprLit, LitBool, LitFloat, LitInt, LitStr};
use toml::value::Array;
use toml::{Table, Value};

#[derive(Clone, Copy)]
enum TableOrArray<'a> {
    Table(&'a Table),
    Array(&'a Array),
}

impl<'a> TableOrArray<'a> {
    fn get(self, segment: &str) -> syn::Result<Option<&'a Value>> {
        Ok(match self {
            TableOrArray::Table(x) => x.get(segment),
            TableOrArray::Array(x) => x.get(segment.parse::<usize>().map_err(|err| {
                Error::new(
                    Span::call_site(),
                    format!("Could not convert key segment '{segment}' into an Array index: {err}"),
                )
            })?),
        })
    }
}

macro_rules! ret {
    ($ty:ident, $val:expr) => {
        return Ok(Expr::Lit(ExprLit {
            attrs: Vec::new(),
            lit: ($ty::new($val, Span::call_site())).into(),
        }))
    };
}

pub fn run(query: ConfigQuery) -> syn::Result<Expr> {
    let table = file::get_config()?;
    let key = query.option.value();
    let parsed = parse_query(&key)?;
    let last = parsed.len() - 1;

    let mut current = TableOrArray::Table(table);
    for (idx, segment) in parsed.into_iter().enumerate() {
        match current.get(segment)? {
            Some(value) => match value {
                Value::String(x) if idx == last => {
                    ret!(LitStr, x)
                }
                Value::Integer(x) if idx == last => {
                    ret!(LitInt, &x.to_string())
                }
                Value::Float(x) if idx == last => {
                    ret!(LitFloat, &x.to_string())
                }
                Value::Boolean(x) if idx == last => {
                    ret!(LitBool, *x)
                }
                Value::Datetime(x) if idx == last => {
                    ret!(LitStr, &x.to_string())
                }
                Value::Array(x) => current = TableOrArray::Array(x),
                Value::Table(x) => current = TableOrArray::Table(x),
                _ => {
                    return Err(Error::new(
                        Span::call_site(),
                        format!("Key '{key}' hat at '{segment}' a non-Table / non-Array value"),
                    ))
                }
            },
            None => {
                return match query.default {
                    Some(x) => Ok(x),
                    None => {
                        return Err(Error::new(
                            Span::call_site(),
                            format!("Key '{key}' has no value in the Config.toml"),
                        ))
                    }
                }
            }
        }
    }

    Err(Error::new(
        Span::call_site(),
        format!("Key '{key}' resolves to a table or Array"),
    ))
}

fn parse_query(mut query: &str) -> syn::Result<Vec<&str>> {
    let mut result = Vec::new();

    while !query.is_empty() {
        if query.starts_with('"') {
            query = parse_until(&query[1..], &mut result, '"')?;
        } else if query.starts_with('\'') {
            query = parse_until(&query[1..], &mut result, '\'')?;
        } else if let Some(idx) = query.find('.') {
            result.push(&query[..idx]);

            query = &query[idx + 1..];
        } else {
            result.push(query);

            break;
        }
    }

    Ok(result)
}

fn parse_until<'a>(
    mut query: &'a str,
    result: &mut Vec<&'a str>,
    end: char,
) -> syn::Result<&'a str> {
    let idx = query.find(end).ok_or_else(|| {
        Error::new(
            Span::call_site(),
            "Quoted key is missing ending quote".to_string(),
        )
    })?;
    result.push(&query[..idx]);

    query = &query[idx + 1..];

    if !query.is_empty() {
        if !matches!(query.bytes().next(), Some(b'.')) {
            return Err(Error::new(
                Span::call_site(),
                "Expected '.' after quoted key".to_string(),
            ));
        }

        Ok(&query[1..])
    } else {
        Ok(query)
    }
}

#[cfg(test)]
mod test {
    use super::{parse_query, run};
    use crate::query::ConfigQuery;
    use proc_macro2::Span;
    use syn::{Expr, LitStr, LitInt, LitBool, LitFloat, ExprLit};

    macro_rules! assert_run {
        (default $input:expr, $default:expr) => {
            let default = Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit: ($default).into(),
            });
            let result = run(ConfigQuery {
                option: LitStr::new($input, Span::call_site()),
                comma: None,
                default: Some(default.clone()),
            }).unwrap();

            assert_eq!(result, default);
        };
        (fail $input:expr) => {
            let result = run(ConfigQuery {
                option: LitStr::new($input, Span::call_site()),
                comma: None,
                default: None,
            });

            if let Ok(result) = result {
                assert!(false, "Expected run to fail, but got: {:?}", result);
            }
        };
        ($input:expr, $result:expr) => {
            let result = run(ConfigQuery {
                option: LitStr::new($input, Span::call_site()),
                comma: None,
                default: None,
            }).unwrap();

            let Expr::Lit(result) = result else {
                assert!(false, "Resolved expr is not a literal.");
                unreachable!()
            };

            assert_eq!(result.lit, ($result).into());
        };
    }

    #[test]
    fn test_run() {
        crate::file::set_config(toml::toml! {
            foo = 5
            bar.baz = true
            bar.string = "Hello World"
            bar.float = 3.1415926535

            [deeply.nested]
            table.value = false
            array = [1, 2, 3]
            table.array = [ { foo = "wow" }, { foo = 4 }, { bar = 69420 } ]
        });

        assert_run!("foo", LitInt::new("5", Span::call_site()));
        assert_run!("bar.baz", LitBool::new(true, Span::call_site()));
        assert_run!("bar.string", LitStr::new("Hello World", Span::call_site()));
        assert_run!("bar.float", LitFloat::new("3.1415926535", Span::call_site()));
        assert_run!("deeply.nested.table.value", LitBool::new(false, Span::call_site()));
        assert_run!("deeply.nested.array.1", LitInt::new("2", Span::call_site()));
        assert_run!("deeply.nested.table.array.0.foo", LitStr::new("wow", Span::call_site()));
        assert_run!("deeply.nested.table.array.2.bar", LitInt::new("69420", Span::call_site()));

        assert_run!(fail "bar");
        assert_run!(fail "deeply.nested");
        assert_run!(fail "deeply.nested.array");
        assert_run!(fail "deeply.nested.table.array");
        
        assert_run!(default "missing", LitStr::new("Rawr", Span::call_site()));
        assert_run!(default "missing.nested.key", LitStr::new("Yip", Span::call_site()));
    }

    macro_rules! assert_query {
        (fail $input:expr) => {
            if let Ok(segments) = parse_query($input) {
                assert!(false, "Expected parsing to fail, but got: {:?}", segments);
            }
        };
        ($input:expr, $( $output:expr ),* $(,)?) => {
            let segments = parse_query($input).unwrap();
            assert_eq!(segments, vec![$( $output ),*]);
        };
    }

    #[test]
    fn test_parse_query() {
        assert_query!("foo", "foo");
        assert_query!("foo.bar", "foo", "bar");
        assert_query!("foo.bar.'baz'", "foo", "bar", "baz");
        assert_query!("foo.bar.\"baz\"", "foo", "bar", "baz");
        assert_query!("foo.'bar'.\"baz\"", "foo", "bar", "baz");
        assert_query!("foo.'bar'.'baz'", "foo", "bar", "baz");
        assert_query!("foo.\"bar\".'baz'", "foo", "bar", "baz");
        assert_query!("'a very'.'complicated key'", "a very", "complicated key");
        assert_query!(
            "'a very'.'complicated key'.'with a \" in it'",
            "a very",
            "complicated key",
            "with a \" in it"
        );
        assert_query!(
            "'a very'.'complicated key'.\"with a ' in it\"",
            "a very",
            "complicated key",
            "with a ' in it"
        );

        assert_query!(fail "'this is miss a quote");
        assert_query!(fail "\"this is miss a quote");
        assert_query!(fail "'this is also invalid''");
        assert_query!(fail "\"this is also invalid\"\"");
    }
}
