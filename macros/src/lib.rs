//! Derives the [`Actionlike`] trait
//
//! This derive macro was inspired by the `strum` crate's `EnumIter` macro.
//! Original source: <https://github.com/Peternator7/strum>,
//! Copyright (c) 2019 Peter Glotfelty under the MIT License

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemImpl};

mod actionlike;
mod typetag;

mod utils;

#[proc_macro_derive(Actionlike)]
pub fn actionlike(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    crate::actionlike::actionlike_inner(&ast).into()
}

#[proc_macro_attribute]
pub fn serde_typetag(_: TokenStream, input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as ItemImpl);

    crate::typetag::expand_serde_typetag(&ast).into()
}
