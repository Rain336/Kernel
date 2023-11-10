use proc_macro::TokenStream;
use query::ConfigQuery;
use quote::ToTokens;
use syn::parse_macro_input;

mod executer;
mod file;
mod query;

#[proc_macro]
pub fn config(input: TokenStream) -> TokenStream {
    let query = parse_macro_input!(input as ConfigQuery);

    let result = match executer::run(query) {
        Ok(result) => result.into_token_stream(),
        Err(err) => err.into_compile_error(),
    };

    result.into()
}
