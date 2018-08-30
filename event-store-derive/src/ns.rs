use derive_enum::derive_enum;
// use derive_struct::derive_struct;
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use quote::__rt::TokenTree::Group;
use std::string::ToString;
use syn::{Attribute, Data, DataEnum, DeriveInput};
use PROC_MACRO_NAME;

pub struct EnumInfo {
    pub enum_namespace: Ident,
    pub item_ident: TokenStream,
    pub enum_body: DataEnum,
    pub variant_idents: Vec<Ident>,
}

impl EnumInfo {
    pub fn new(input: &DeriveInput, enum_body: &DataEnum) -> Self {
        let enum_namespace = get_namespace_from_attributes(&input.attrs)
            .expect("Namespace attribute must be provided at the enum level");

        let item_ident = input.clone().ident.into_token_stream();

        let variant_idents = enum_body
            .variants
            .iter()
            .map(|v| v.ident.clone())
            .collect::<Vec<Ident>>();

        Self {
            enum_namespace,
            item_ident,
            variant_idents,
            enum_body: enum_body.clone(),
        }
    }
}

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

pub fn get_enum_struct_names(enum_body: &DataEnum) -> Vec<TokenStream> {
    enum_body
        .variants
        .iter()
        .map(|variant| {
            variant
                .fields
                .iter()
                .next()
                .map(|field| field.ty.clone().into_token_stream())
                .expect("Expected struct type")
        }).collect::<Vec<TokenStream>>()
}

pub fn expand_derive_namespace(parsed: &DeriveInput) -> TokenStream {
    match parsed.data {
        Data::Enum(ref body) => derive_enum(&parsed, &body),
        // Data::Struct(ref body) => derive_struct(&parsed, &body),
        _ => panic!("Namespace can only be derived on enums"),
    }
}

// Resolve and stringify a list of namespaces for all fields in an enum
pub fn get_quoted_namespaces(enum_body: &DataEnum, default_namespace: &Ident) -> Vec<String> {
    enum_body
        .variants
        .iter()
        .map(|variant| {
            get_namespace_from_attributes(&variant.attrs)
                .unwrap_or(default_namespace.clone())
                .to_string()
        }).collect()
}
