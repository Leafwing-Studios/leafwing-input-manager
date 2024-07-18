use crate::utils;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// This approach and implementation is inspired by the `strum` crate,
/// Copyright (c) 2019 Peter Glotfelty
/// available under the MIT License at <https://github.com/Peternator7/strum>

pub(crate) fn actionlike_inner(ast: &DeriveInput) -> TokenStream {
    // Splitting the abstract syntax tree
    let enum_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let crate_path = utils::crate_path();

    quote! {
        impl #impl_generics #crate_path::Actionlike for #enum_name #type_generics #where_clause {
            fn input_control_kind(&self) -> #crate_path::InputControlKind {
                    #crate_path::InputControlKind::Button
            }
        }
    }
}
