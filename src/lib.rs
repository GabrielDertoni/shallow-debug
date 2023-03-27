//! # Shallow debug
//!
//! A crate that allows any type to derive a very simple and "shallow" debug impl. The impl will
//! only print the enum variant, but not the content of the variant. For structs, it will only
//! print the struct's name, and none of it's field's values.
//!
//! This is mainly useful for enums when the variant is already useful information. Since none of
//! the inner values are printed, they don't have to implement `Debug`, so this can also be useful
//! in highly generic code where you just want a quick and simple way to get debug information.
//!
//! ## Example
//!
//! ```rust
//! # use shallow_debug::ShallowDebug;
//! #[derive(ShallowDebug)]
//! enum MyEnum<A, B, C> {
//!     A(A),
//!     B(B),
//!     C(C),
//! }
//!
//! let value: MyEnum<i32, &str, usize> = MyEnum::A(123);
//! assert_eq!(format!("{value:?}"), "MyEnum::A(..)");
//! ```

use syn::{Data, Fields, GenericParam};
use quote::{quote, ToTokens};

/// A derive macro that is able to implement `Debug` for any type, without requiring it's inner
/// types to also implement the `Debug` trait. In order to do this, the `Debug` impl that is
/// generated is "shallow", meaning it will only print the enum variant names, but not their
/// internal values. You can also `#[derive(ShallowDebug)]` for structs and unions, but it will not
/// print the field values. In general this is more useful for enums, since the variant can
/// already tell you useful information.
#[proc_macro_derive(ShallowDebug)]
pub fn derive_shallow_debug(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(stream as syn::DeriveInput);

    let ident = &input.ident;
    let fmt_body = match &input.data {
        Data::Enum(data_enum) => {
            let variants = data_enum.variants.iter()
                .map(|variant| {
                    let variant_ident = &variant.ident;
                    match &variant.fields {
                        Fields::Named(_) => {
                            let fmt = format!("{ident}::{variant_ident}{{{{..}}}}");
                            quote!(#ident::#variant_ident{..} => write!(f, #fmt))
                        }
                        Fields::Unnamed(_) => {
                            let fmt = format!("{ident}::{variant_ident}(..)");
                            quote!(#ident::#variant_ident(..) => write!(f, #fmt))
                        }
                        Fields::Unit => {
                            let fmt = format!("{ident}::{variant_ident}");
                            quote!(#ident::#variant_ident => write!(f, #fmt))
                        }
                    }
                });

            quote! {
                match self {
                    #(#variants,)*
                }
            }
        }
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(_) => {
                let fmt = format!("{ident}{{{{..}}}}");
                quote!(write!(f, #fmt))
            }
            Fields::Unnamed(_) => {
                let fmt = format!("{ident}(..)");
                quote!(write!(f, #fmt))
            }
            Fields::Unit => {
                let fmt = format!("{ident}");
                quote!(write!(f, #fmt))
            }
        }

        Data::Union(_) => {
            let fmt = format!("{ident}");
            quote!(write!(f, #fmt))
        }
    };

    let bounds = input.generics.params.iter()
        .filter_map(|param| match param {
            GenericParam::Lifetime(lifetime) if lifetime.bounds.is_empty() => None,
            GenericParam::Lifetime(lifetime) => {
                let bounds = &lifetime.bounds;
                let ident = &lifetime.lifetime;
                Some(quote!(#ident: #bounds))
            }
            GenericParam::Type(ty) if ty.bounds.is_empty() => None,
            GenericParam::Type(ty) => {
                let bounds = &ty.bounds;
                let ident = &ty.ident;
                Some(quote!(#ident: #bounds))
            }
            GenericParam::Const(_) => None,
        })
        .chain({
            input.generics.where_clause.iter()
                .flat_map(|clause| clause.predicates.iter().map(ToTokens::to_token_stream))
        });

    let ty_vars = input.generics.params.iter()
        .map(|param| match param {
            GenericParam::Lifetime(lifetime) => lifetime.lifetime.to_token_stream(),
            GenericParam::Type(ty) => {
                let ident = &ty.ident;
                if let Some(default) = &ty.default {
                    quote!(#ident = #default)
                } else {
                    ident.to_token_stream()
                }
            }
            GenericParam::Const(cons) => cons.to_token_stream(),
        })
        .collect::<Vec<_>>();

    // The `impl Debug for <type> where ...` part
    let impl_debug = if ty_vars.is_empty() {
        quote! {
            impl std::fmt::Debug for #ident
        }
    } else {
        quote! {
            impl<#(#ty_vars),*> std::fmt::Debug for #ident<#(#ty_vars),*>
            where
                #(#bounds),*
        }
    };

    quote! {
        #impl_debug {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #fmt_body
            }
        }
    }.into()
}

