use crate::PROC_MACRO_NAME;
use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use syn::{Attribute, DataEnum, Lit, Meta, NestedMeta};

/// Attributes taken from `#[derive()]` statement on an enum variant
#[derive(Default, Debug)]
pub(crate) struct VariantEventStoreAttributes {
    /// Event type like `ThingUpdated`
    pub event_type: String,

    /// Event namespace override from enum definition
    ///
    /// Unused
    pub event_namespace: Option<String>,

    /// Entity override from enum definition
    ///
    /// Unused
    pub entity_type: Option<String>,
}

pub(crate) struct EnumEventStoreAttributes {
    /// Event namespace like `accounts` or `organisations`
    pub event_namespace: String,

    /// Entity type like `user` or `organisation`
    pub entity_type: String,
}

pub(crate) struct EnumExt<'a> {
    pub ident: Ident,
    pub enum_body: &'a DataEnum,
    pub event_store_attributes: EnumEventStoreAttributes,
}

pub fn attributes_map(attrs: &Vec<Attribute>) -> Result<HashMap<String, String>, String> {
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
