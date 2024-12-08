use crate::utils;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::BTreeMap;
use syn::{Attribute, Data, DataEnum, DeriveInput, Error, Ident};

// This approach and implementation is inspired by the `strum` crate,
// Copyright (c) 2019 Peter Glotfelty
// available under the MIT License at <https://github.com/Peternator7/strum>

pub(crate) fn actionlike_inner(ast: &DeriveInput) -> syn::Result<TokenStream> {
    // Splitting the abstract syntax tree
    let enum_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let crate_path = utils::crate_path();

    let default_control = parse_default_control(ast)?;
    let input_control_kind_body =
        generate_input_control_kind_body(ast, &crate_path, &default_control)?;
    Ok(quote! {
        impl #impl_generics #crate_path::Actionlike for #enum_name #type_generics #where_clause {
            fn input_control_kind(&self) -> #crate_path::InputControlKind {
                #input_control_kind_body
            }
        }
    })
}

fn parse_default_control(ast: &DeriveInput) -> syn::Result<Ident> {
    if let Some(attr) = ast
        .attrs
        .iter()
        .find(|attr| attr.meta.path().is_ident("actionlike"))
    {
        parse_control_attr(attr)
    } else {
        Ok(Ident::new("Button", Span::call_site()))
    }
}

fn parse_control_attr(attr: &Attribute) -> syn::Result<Ident> {
    attr.meta
        .require_list()
        .and_then(|list| list.parse_args::<Ident>())
        .map_err(|_| {
            let span = quote!(#attr);
            let msg = "expected only one item like `#[actionlike(Button)]`";
            Error::new_spanned(span, msg)
        })
}

fn generate_input_control_kind_body(
    ast: &DeriveInput,
    crate_path: &TokenStream,
    default_control: &Ident,
) -> Result<TokenStream, Error> {
    match &ast.data {
        Data::Enum(enum_data) => {
            // Gather variants whose control kinds deviate from the default.
            let controls = parse_variant_controls(enum_data, default_control)?;
            if controls.is_empty() {
                return Ok(quote!(#crate_path::InputControlKind::#default_control));
            }

            let controls: Vec<_> = controls
                .iter()
                .map(|(variant, control)| quote!(Self::#variant => #crate_path::InputControlKind::#control,))
                .collect();
            Ok(quote! {
                match self {
                    #(#controls)*
                    _ => #crate_path::InputControlKind::#default_control,
                }
            })
        }
        _ => Ok(quote!(#crate_path::InputControlKind::#default_control)),
    }
}

fn parse_variant_controls(
    data: &DataEnum,
    default_control: &Ident,
) -> syn::Result<BTreeMap<Ident, Ident>> {
    let mut map = BTreeMap::<Ident, Ident>::new();
    for variant in data.variants.iter() {
        for attr in variant
            .attrs
            .iter()
            .filter(|attr| attr.meta.path().is_ident("actionlike"))
        {
            let control = parse_control_attr(attr)?;
            if &control != default_control {
                map.insert(variant.ident.clone(), control);
            }
        }
    }
    Ok(map)
}
