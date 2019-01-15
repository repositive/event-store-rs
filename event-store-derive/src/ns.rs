use crate::derive_enum::derive_enum;
use crate::derive_struct::derive_struct;
use crate::PROC_MACRO_NAME;
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;
use quote::__rt::TokenTree::Group;
use std::string::ToString;
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed};

pub struct EnumInfo {
    pub item_ident: TokenStream,
    pub enum_body: DataEnum,
    pub variant_idents: Vec<Ident>,
    pub renamed_variant_idents: Vec<Ident>,
}

impl EnumInfo {
    pub fn new(input: &DeriveInput, enum_body: &DataEnum) -> Self {
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
            item_ident,
            variant_idents,
            renamed_variant_idents,
            enum_body: enum_body.clone(),
        }
    }
}

pub struct StructInfo {
    pub field_idents: Vec<Ident>,
    pub fields: FieldsNamed,
    pub item_ident: Ident,
    pub item_ident_quoted: String,
    pub namespace_and_type: String,
    pub renamed_item_ident: Ident,
    pub renamed_item_ident_quoted: String,
    pub renamed_namespace_and_type: String,
    pub struct_body: DataStruct,
    pub struct_namespace: Ident,
    pub struct_namespace_quoted: String,
}

impl StructInfo {
    pub fn new(parsed: &DeriveInput, struct_body: &DataStruct) -> Self {
        let struct_namespace = get_attribute_ident(&parsed.attrs, "namespace")
            .expect("Namespace attribute must be provided at the struct level");

        let struct_rename = get_attribute_ident(&parsed.attrs, "rename");

        let item_ident = parsed.clone().ident;
        let renamed_item_ident = struct_rename.unwrap_or(item_ident.clone());

        let fields = match struct_body.fields {
            Fields::Named(ref f) => f.clone(),
            _ => panic!("Store derive only supports structs with named fields"),
        };

        let field_idents = struct_body
            .fields
            .iter()
            .filter_map(|field| field.clone().ident)
            .collect::<Vec<Ident>>();

        let namespace_and_type = format!("{}.{}", struct_namespace, item_ident);
        let renamed_namespace_and_type = format!("{}.{}", struct_namespace, renamed_item_ident);

        let item_ident_quoted = item_ident.to_string();
        let struct_namespace_quoted = struct_namespace.to_string();
        let renamed_item_ident_quoted = renamed_item_ident.to_string();

        Self {
            field_idents,
            fields,
            item_ident,
            item_ident_quoted,
            namespace_and_type,
            renamed_item_ident,
            renamed_item_ident_quoted,
            renamed_namespace_and_type,
            struct_body: struct_body.clone(),
            struct_namespace,
            struct_namespace_quoted,
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
                            ) if *ident == ident_match => Some(Ident::new(
                                attribute_value.to_string().trim_matches('"').into(),
                                Span::call_site(),
                            )),
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
        Data::Struct(ref body) => derive_struct(&parsed, &body),
        _ => panic!("Namespace can only be derived on enums and structs"),
    }
}
