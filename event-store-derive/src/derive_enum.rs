use ns::get_namespace_from_attributes;
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use std::iter::repeat;
use std::str::FromStr;
use syn::{DataEnum, DeriveInput};

pub fn derive_enum(parsed: &DeriveInput, enum_body: &DataEnum) -> TokenStream {
    let default_namespace = get_namespace_from_attributes(&parsed.attrs)
        .expect("Namespace attribute must be provided at the enum level");

    let item_ident = parsed.clone().ident.into_token_stream();
    let item_idents = repeat(&item_ident);

    let variant_idents = enum_body.variants.iter().map(|v| v.ident.clone());
    let variant_namespaces = enum_body.variants.iter().map(|variant| {
        get_namespace_from_attributes(&variant.attrs).unwrap_or(default_namespace.clone())
    });

    let namespaced_variants_quoted = variant_namespaces
        .zip(enum_body.variants.iter())
        .map(|(ns, variant)| {
            TokenStream::from_str(&format!("\"{}.{}\"", ns, variant.ident)).expect("Variant name")
        }).collect::<Vec<TokenStream>>();

    let out = quote!{
        impl event_store_derive_internals::EventData for #item_ident {
            fn namespaced_type(&self) -> &'static str {
                match self {
                    #(#item_idents::#variant_idents(_) => #namespaced_variants_quoted,)*
                }
            }
        }

        // #ser

        // #de
    };

    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_ENUM_FOR_{}", item_ident),
        Span::call_site(),
    );

    quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            // extern crate serde as serde;
            extern crate event_store_derive_internals;
            #out
        };
    }
}
