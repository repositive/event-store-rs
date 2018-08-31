use derive_enum::derive_enum;
// use derive_struct::derive_struct;
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
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
    pub renamed_variant_idents: Vec<Ident>,
}

impl EnumInfo {
    pub fn new(input: &DeriveInput, enum_body: &DataEnum) -> Self {
        let enum_namespace = get_attribute_ident(&input.attrs, "namespace")
            .expect("Namespace attribute must be provided at the enum level");

        let item_ident = input.clone().ident.into_token_stream();

        let variant_idents = enum_body
            .variants
            .iter()
            .map(|v| v.ident.clone())
            .collect::<Vec<Ident>>();

        let renamed_variant_idents = enum_body
            .variants
            .iter()
            .map(|v| {
                let name_override = get_attribute_ident(&v.attrs, "rename");

                name_override.unwrap_or(v.ident.clone())
            })
            .collect::<Vec<Ident>>();

        Self {
            enum_namespace,
            item_ident,
            variant_idents,
            renamed_variant_idents,
            enum_body: enum_body.clone(),
        }
    }
}

pub fn get_attribute_ident(input: &Vec<Attribute>, attribute_name: &'static str) -> Option<Ident> {
    let ident_match = Ident::new(attribute_name, Span::call_site());

    input
        .iter()
        .filter_map(|attr| {
            // Look through all attribute annotations
            attr.path
                .segments
                .iter()
                .find(|segment| segment.ident.to_string() == PROC_MACRO_NAME)
                .and_then(|_| {
                    // Find attribute triples like `namespace = "something"`
                    attr.clone().tts.into_iter().find(|tt| match tt {
                        Group(_) => true,
                        _ => false,
                    })
                })
                .and_then(|tt| match tt {
                    Group(g) => {
                        let mut it = g.stream().into_iter();

                        match (it.nth(0), it.nth(1)) {
                            (
                                Some(TokenTree::Ident(ref ident)),
                                Some(TokenTree::Literal(ref attribute_value)),
                            ) if *ident == ident_match =>
                            {
                                Some(Ident::new(
                                    attribute_value.to_string().trim_matches('"').into(),
                                    Span::call_site(),
                                ))
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                })
        })
        .next()
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
        })
        .collect::<Vec<TokenStream>>()
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
            get_attribute_ident(&variant.attrs, "namespace")
                .unwrap_or(default_namespace.clone())
                .to_string()
        })
        .collect()
}
