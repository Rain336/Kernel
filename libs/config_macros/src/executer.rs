use crate::{file, ConfigQuery};
use proc_macro::Span;
use syn::{Error, Expr, ExprLit, Lit, LitBool, LitFloat, LitInt, LitStr};
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
                    Span::call_site().into(),
                    format!("Could not convert key segment '{segment}' into an Array index: {err}"),
                )
            })?),
        })
    }
}

macro_rules! ret {
    ($e:path, $ty:path, $val:expr) => {
        return Ok(Expr::Lit(ExprLit {
            attrs: Vec::new(),
            lit: $e($ty($val, proc_macro::Span::call_site().into())),
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
                    ret!(Lit::Str, LitStr::new, x)
                }
                Value::Integer(x) if idx == last => {
                    ret!(Lit::Int, LitInt::new, &x.to_string())
                }
                Value::Float(x) if idx == last => {
                    ret!(Lit::Float, LitFloat::new, &x.to_string())
                }
                Value::Boolean(x) if idx == last => {
                    ret!(Lit::Bool, LitBool::new, *x)
                }
                Value::Datetime(x) if idx == last => {
                    ret!(Lit::Str, LitStr::new, &x.to_string())
                }
                Value::Array(x) => current = TableOrArray::Array(x),
                Value::Table(x) => current = TableOrArray::Table(x),
                _ => {
                    return Err(Error::new(
                        Span::call_site().into(),
                        format!("Key '{key}' hat at '{segment}' a non-Table / non-Array value"),
                    ))
                }
            },
            None => {
                return match query.default {
                    Some(x) => Ok(x),
                    None => {
                        return Err(Error::new(
                            Span::call_site().into(),
                            format!("Key '{key}' has no value in the Config.toml"),
                        ))
                    }
                }
            }
        }
    }

    Err(Error::new(
        Span::call_site().into(),
        format!("Key '{key}' resolves to a table or Array"),
    ))
}

fn parse_query(mut query: &str) -> syn::Result<Vec<&str>> {
    let mut result = Vec::new();

    loop {
        if query.starts_with(['"', '\'']) {
            let idx = query[1..].find(['"', '\'']).ok_or_else(|| {
                Error::new(
                    Span::call_site().into(),
                    "Quoted key is missing ending quote".to_string(),
                )
            })?;
            result.push(&query[1..idx - 1]);

            if !matches!(query[..idx + 1].bytes().next(), Some(b'.')) {
                return Err(Error::new(
                    Span::call_site().into(),
                    "Expected '.' after quoted key".to_string(),
                ));
            }

            query = &query[..idx + 2];
        } else if let Some(idx) = query.find('.') {
            result.push(&query[..idx]);

            query = &query[..idx + 1];
        } else {
            result.push(query);

            break;
        }
    }

    Ok(result)
}
