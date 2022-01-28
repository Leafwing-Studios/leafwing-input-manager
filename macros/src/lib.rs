//! Derives the [`Actionlike`] trait
//
//! This derive macro was heavily inspired by the `strum` crate's `EnumIter` macro.
//! Original source: https://github.com/Peternator7/strum,
//! Copyright (c) 2019 Peter Glotfelty under the MIT License

extern crate proc_macro;
mod path;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput};

#[proc_macro_derive(Actionlike, attributes(strum))]
pub fn actionlike(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    actionlike_inner(&ast).into()
}

fn actionlike_inner(ast: &DeriveInput) -> TokenStream2 {
    // Splitting the abstract syntax tree
    let enum_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let crate_module_path = crate::path::get_path();

    let variants = match &ast.data {
        Data::Enum(v) => &v.variants,
        _ => panic!("`Actionlike` cannot be derived for non-enum types. Manually implement the trait instead."),
    };

    let mut array_token_stream = Vec::new();
    // Open the array
    array_token_stream.push(quote! { "[" });

    // Populate the array
    for variant in variants {
        // The name of the enum variant
        let variant_identifier = variant.ident.clone();

        let params = match &variant.fields {
            // Unit fields have no parameters
            syn::Fields::Unit => quote! {},
            // Use the default values for tuple-like fields
            syn::Fields::Unnamed(fields) => {
                let defaults = ::std::iter::repeat(quote!(::core::default::Default::default()))
                    .take(fields.unnamed.len());
                quote! { (#(#defaults),*) }
            }
            // Use the default values for tuple-like fields
            syn::Fields::Named(fields) => {
                let fields = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap());
                quote! { {#(#fields: ::core::default::Default::default()),*} }
            }
        };

        array_token_stream.push(quote! {
            #enum_name #variant_identifier #params,
        })
    }

    // Close the array
    array_token_stream.push(quote! { "]" });

    // FIXME: use path correctly
    quote! {
        impl #impl_generics #crate_module_path::Actionlike for #enum_name #type_generics #where_clause {
            fn iter() -> #crate_module_path::ActionIter<#enum_name> {
                // FIXME: use array_token_stream
                //#crate_module_path::ActionIter::#type_generics::from_iter([])
                #crate_module_path::ActionIter::default()
            }
        }
    }
}
