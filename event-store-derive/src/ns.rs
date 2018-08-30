use derive_enum::derive_enum;
use derive_struct::derive_struct;
use proc_macro2::{Ident, Span, TokenStream};
use quote::__rt::TokenTree::Group;
use std::string::ToString;
use syn::{Attribute, Data, DeriveInput};
use PROC_MACRO_NAME;

pub fn get_namespace_from_attributes(input: &Vec<Attribute>) -> Option<Ident> {
    input
        .iter()
        .filter_map(|attr| {
            // Look through all attribute annotations
            attr.path
                .segments
                .iter()
                // Filter attributes we're interested in
                .find(|segment| segment.ident.to_string() == PROC_MACRO_NAME)
                // Find attribute triples like `namespace = "something"`
                .and_then(|_| {
                    attr.clone().tts.into_iter().find(|tt| match tt {
                        Group(_) => true,
                        _ => false,
                    })
                }).and_then(|tt| match tt {
                    // Get last token of `a = b` triplet
                    Group(g) => g
                        .stream()
                        .into_iter()
                        .nth(2)
                        .map(|namespace| Ident::new(namespace.to_string().trim_matches('"').into(), Span::call_site(),)),
                    _ => None,
                })
        }).next()
}

pub fn expand_derive_namespace(parsed: &DeriveInput) -> TokenStream {
    match parsed.data {
        Data::Enum(ref body) => derive_enum(&parsed, &body),
        // Data::Struct(ref body) => derive_struct(&parsed, &body),
        _ => panic!("Namespace can only be derived on enums"),
    }
}
