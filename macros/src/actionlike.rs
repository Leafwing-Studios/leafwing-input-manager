use crate::utils;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// This approach and implementation is inspired by the `strum` crate,
/// Copyright (c) 2019 Peter Glotfelty
/// available under the MIT License at <https://github.com/Peternator7/strum>

fn parse_field_type(field: &syn::Field) -> &'static str {
    let field_type = &field.ty;
    if quote!(#field_type).to_string() == quote!(f32).to_string() {
        "Axis"
    } else if quote!(#field_type).to_string() == quote!(bevy::math::Vec2).to_string() {
        "DualAxis"
    } else {
        panic!("Actionlike can only be derived for enums with f32 or Vec2 fields");
    }
}

/// Parses a variant and returns the kind of input control it represents
///
/// A variant is considered buttonlike if it holds no values, axislike if it holds a single f32 value, and dual-axislike if it holds a Vec2 value.
/// All other variants are considered buttonlike.
fn compute_control_kind_for_variant(variant: &syn::Variant) -> TokenStream {
    let crate_path = utils::crate_path();
    let control_kind = match &variant.fields {
        syn::Fields::Unit => "Button",
        syn::Fields::Named(named_fields) => {
            if named_fields.named.len() == 1 {
                let field = named_fields.named.iter().next().unwrap();
                parse_field_type(field)
            } else {
                panic!("Actionlike can only be derived for enums with unit or single-field named variants");
            }
        }
        syn::Fields::Unnamed(unnamed_fields) => {
            if unnamed_fields.unnamed.len() == 1 {
                let field = unnamed_fields.unnamed.iter().next().unwrap();
                parse_field_type(field)
            } else {
                panic!("Actionlike can only be derived for enums with unit or single-field named variants");
            }
        }
    };

    quote! {
        #crate_path::InputControlKind::#control_kind
    }
}

pub(crate) fn actionlike_inner(ast: &DeriveInput) -> TokenStream {
    // Splitting the abstract syntax tree
    let enum_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let crate_path = utils::crate_path();

    let control_kind_match_arms = match &ast.data {
        syn::Data::Enum(data) => data.variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            let control_kind = compute_control_kind_for_variant(variant);
            quote! {
                #enum_name::#variant_name => #crate_path::InputControlKind::#control_kind
            }
        }),
        _ => panic!("Actionlike can only be derived for enums"),
    };

    quote! {
        impl #impl_generics #crate_path::Actionlike for #enum_name #type_generics #where_clause {
            fn input_control_kind(&self) -> #crate_path::InputControlKind {
                match self {
                    #(#control_kind_match_arms),*
                }
            }
        }
    }
}
