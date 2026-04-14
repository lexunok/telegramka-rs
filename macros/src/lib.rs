use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, ItemFn, Pat, Type, parse_macro_input};

fn find_claims_arg(input: &mut ItemFn) -> syn::Result<(Ident, Box<Type>)> {
    let claims_arg = input.sig.inputs.iter_mut().find_map(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            if let Type::Path(type_path) = &*pat_type.ty {
                if type_path
                    .path
                    .segments
                    .last()
                    .map_or(false, |segment| segment.ident == "Claims")
                {
                    return Some(pat_type);
                }
            }
        }
        None
    });

    if let Some(pat_type) = claims_arg {
        if let Pat::Ident(pat_ident) = &*pat_type.pat {
            Ok((pat_ident.ident.clone(), pat_type.ty.clone()))
        } else {
            Err(syn::Error::new_spanned(
                pat_type,
                "Expected ident pattern for Claims argument",
            ))
        }
    } else {
        Err(syn::Error::new_spanned(
            input.sig.ident.clone(),
            "Handler function must have a `claims: Claims` argument",
        ))
    }
}

#[proc_macro_derive(IntoDataResponse)]
pub fn into_data_response_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;
    let generated_impl = quote! {
        impl axum::response::IntoResponse for #name {
            fn into_response(self) -> axum::response::Response {
                axum::Json(self).into_response()
            }
        }
    };
    generated_impl.into()
}
