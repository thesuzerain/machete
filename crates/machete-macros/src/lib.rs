use proc_macro::{self, TokenStream};

mod filter;

#[proc_macro_derive(Filterable, attributes(filter))]
pub fn filterable(input: TokenStream) -> TokenStream {
    filter::derive_proc_macro_impl(input)
}
