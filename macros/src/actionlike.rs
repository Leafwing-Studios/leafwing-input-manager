use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub(crate) fn actionlike_inner(ast: &DeriveInput) -> TokenStream {
    // Splitting the abstract syntax tree
    let enum_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let crate_path = crate::path::get_path();

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

    // FIXME: use real output
    array_token_stream = vec![quote! {"["}, quote! {"]"}];

    quote! {
        impl #impl_generics #crate_path::Actionlike for #enum_name #type_generics #where_clause {
            fn iter() -> #crate_path::ActionIter<#enum_name> {
                //#crate_path::ActionIter::from_iter(#(#array_token_stream)*)
                #crate_path::ActionIter::default()
            }
        }
    }
}
