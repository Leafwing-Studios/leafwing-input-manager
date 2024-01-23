use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{DeriveInput, Ident};

/// This approach and implementation is inspired by the `strum` crate,
/// Copyright (c) 2019 Peter Glotfelty
/// available under the MIT License at <https://github.com/Peternator7/strum>

pub(crate) fn actionlike_inner(ast: &DeriveInput) -> TokenStream {
    // Splitting the abstract syntax tree
    let enum_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let crate_path = if let Ok(found_crate) = crate_name("leafwing_input_manager") {
        // The crate was found in the Cargo.toml
        match found_crate {
            FoundCrate::Itself => quote!(leafwing_input_manager),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote!(#ident)
            }
        }
    } else {
        // The crate was not found in the Cargo.toml,
        // so we assume that we are in the owning_crate itself
        //
        // In order for this to play nicely with unit tests within the crate itself,
        // `use crate as leafwing_input_manager` at the top of each test module
        //
        // Note that doc tests, integration tests and examples want the full standard import,
        // as they are evaluated as if they were external
        quote!(leafwing_input_manager)
    };

    quote! {
        impl #impl_generics #crate_path::Actionlike for #enum_name #type_generics #where_clause {}
    }
}
