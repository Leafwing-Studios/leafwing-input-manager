extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Actionlike)]
pub fn derive_actionlike(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    // Build the output
    let expanded = quote! {
        impl Actionlike for #struct_name {}
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
