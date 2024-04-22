use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, ItemImpl, Type, TypePath};

use crate::utils;

/// This approach and implementation is inspired by the `typetag` crate,
/// Copyright (c) 2019 David Tolnay
/// available under either of `Apache License, Version 2.0` or `MIT` license
/// at <https://github.com/dtolnay/typetag>
pub(crate) fn expand_serde_typetag(input: &ItemImpl) -> TokenStream {
    let Some(trait_) = &input.trait_ else {
        let impl_token = input.impl_token;
        let ty = &input.self_ty;
        let span = quote!(#impl_token, #ty);
        let msg = "expected impl Trait for Type";
        return Error::new_spanned(span, msg).to_compile_error();
    };

    let trait_path = &trait_.1;

    let where_clause = &input.generics.where_clause;
    let generics_params = &input.generics.params;

    let self_ty = &input.self_ty;

    let ident = match type_name(self_ty) {
        Some(name) => quote!(#name),
        None => {
            let impl_token = input.impl_token;
            let span = quote!(#impl_token, #self_ty);
            let msg = "expected explicit name for Type";
            return Error::new_spanned(span, msg).to_compile_error();
        }
    };

    let crate_path = utils::crate_path();

    quote! {
        #input

        impl<'de, #generics_params> #crate_path::typetag::RegisterTypeTag<'de, dyn #trait_path> for #self_ty #where_clause {
            fn register_typetag(
                registry: &mut #crate_path::typetag::MapRegistry<dyn #trait_path>,
            ) {
                #crate_path::typetag::Registry::register(
                    registry,
                    #ident,
                    |de| Ok(::std::boxed::Box::new(
                        ::bevy::reflect::erased_serde::deserialize::<#self_ty>(de)?,
                    )),
                )
            }
        }
    }
}

fn type_name(mut ty: &Type) -> Option<String> {
    loop {
        match ty {
            Type::Group(group) => {
                ty = &group.elem;
            }
            Type::Path(TypePath { qself, path }) if qself.is_none() => {
                return Some(path.segments.last().unwrap().ident.to_string())
            }
            _ => return None,
        }
    }
}
