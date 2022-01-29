use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{Data, DeriveInput, Ident};

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

    let variants = match &ast.data {
        Data::Enum(v) => &v.variants,
        _ => panic!("`Actionlike` cannot be derived for non-enum types. Manually implement the trait instead."),
    };

    // Populate the array
    let mut array_token_stream = Vec::new();
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

        // Enum variant
        array_token_stream.push(quote! {
            #enum_name::#variant_identifier #params,
        });
    }

    quote! {
        impl #impl_generics #crate_path::Actionlike for #enum_name #type_generics #where_clause {
            fn iter() -> #crate_path::ActionIter<#enum_name> {
                #crate_path::ActionIter::from_iter([#(#array_token_stream)*])
            }
        }
    }
}
