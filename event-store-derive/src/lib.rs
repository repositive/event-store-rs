#![recursion_limit = "128"]

extern crate proc_macro;

mod derive_enum;
mod enum_helpers;

use crate::derive_enum::derive_enum;
use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use syn::{Attribute, Data, DeriveInput, Lit, Meta, NestedMeta};

/// Name of attribute used in `#[derive()]` statements
const PROC_MACRO_NAME: &'static str = "event_store";

#[proc_macro_derive(CreateEvents, attributes(event_store))]
pub fn derive_create_events(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    match input.data {
        Data::Enum(ref body) => derive_enum(
            &input,
            &body,
            Ident::new("EventStoreCreateEvents", Span::call_site()),
        ),
        _ => panic!("Entity create events must be an enum"),
    }
    .into()
}

#[proc_macro_derive(UpdateEvents, attributes(event_store))]
pub fn derive_update_events(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    match input.data {
        Data::Enum(ref body) => derive_enum(
            &input,
            &body,
            Ident::new("EventStoreUpdateEvents", Span::call_site()),
        ),
        _ => panic!("Entity update events must be an enum"),
    }
    .into()
}

fn attributes_map(attrs: &Vec<Attribute>) -> Result<HashMap<String, String>, String> {
    let ident = Ident::new(PROC_MACRO_NAME, Span::call_site());

    attrs
        .iter()
        // Find only attributes called `event_store`
        .find(|attr| attr.path.is_ident(ident.clone()))
        .ok_or(format!(
            "Failed to find attribute {} for {}",
            PROC_MACRO_NAME, ident
        ))
        // Parse metadata
        .and_then(|event_store_attr| event_store_attr.parse_meta().map_err(|e| e.to_string()))
        // Get list of meta key/value paris
        .and_then(|meta| match meta {
            // Metadata must be a [list](https://docs.rs/syn/0.15.34/syn/enum.Meta.html#list)
            Meta::List(meta_key_values) => {
                meta_key_values
                    .nested
                    .iter()
                    .map(|item| match item {
                        // Metadata item in this list must be a `name = "value"` pair
                        NestedMeta::Meta(Meta::NameValue(name_value)) => {
                            let name = name_value.ident.to_string();

                            // The value of this pair must be a string, as that's all that is
                            // supported by event_store_derive right now.
                            match &name_value.lit {
                                Lit::Str(lit) => Ok((name, lit.value().clone())),
                                _ => Err(format!("Value for property {} must be a string", name)),
                            }
                        }
                        _ => Err(format!(
                            "Attribute properties must be a list of key/value pairs"
                        )),
                    })
                    .collect::<Result<HashMap<String, String>, String>>()
            }
            _ => Err(format!(
                "Metadata must be a list like 'event_namespace = \"foo_bar\"'"
            )),
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
