use ns::get_namespace_from_attributes;
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use std::collections::HashMap;
use std::iter::repeat;
use std::str::FromStr;
use std::string::ToString;
use syn::{DataEnum, DeriveInput};

pub fn derive_enum(parsed: &DeriveInput, body: &DataEnum) -> TokenStream {
    let default_namespace = get_namespace_from_attributes(&parsed.attrs)
        .expect("Namespace attribute must be provided at the enum level");

    let variants: HashMap<String, String> = body
        .variants
        .iter()
        .map(|variant| {
            let variant_namespace_override = get_namespace_from_attributes(&variant.attrs);

            let variant_namespace = variant_namespace_override.unwrap_or(default_namespace.clone());

            (variant.ident.to_string(), variant_namespace)
        }).collect();

    let item_ident = parsed.clone().ident.into_token_stream();
    let item_idents = repeat(&item_ident);

    let variant_names = variants
        .keys()
        .map(|k| TokenStream::from_str(k).expect("Variant name"));

    let namespaced_variants = variants.iter().map(|(ident, ns)| {
        TokenStream::from_str(&format!("\"{}.{}\"", ns, ident)).expect("Variant name")
    });

    let out = quote!{
        impl event_store_derive_internals::EventData for #item_ident {
            fn namespaced_type(&self) -> &'static str {
                match self {
                    #(#item_idents::#variant_names(_) => #namespaced_variants,)*
                }
            }
        }
    };

    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_FOR_{}", item_ident),
        Span::call_site(),
    );

    quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate serde as serde;
            extern crate event_store_derive_internals;
            #out
        };
    }
}
