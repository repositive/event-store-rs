use crate::PROC_MACRO_NAME;
use proc_macro2::{Ident, Span, TokenStream};
use std::collections::HashMap;

use quote::{quote, ToTokens};

use syn::{Attribute, DataEnum, DeriveInput, Lit, Meta, NestedMeta, Variant};

/// Attributes taken from `#[derive()]` statement on an enum variant
#[derive(Default, Debug)]
struct VariantEventStoreAttributes {
    /// Event type like `ThingUpdated`
    event_type: String,

    /// Event namespace override from enum definition
    ///
    /// Unused
    event_namespace: Option<String>,

    /// Entity override from enum definition
    ///
    /// Unused
    entity_type: Option<String>,
}

struct VariantExt<'a> {
    variant: &'a Variant,
    event_store_attributes: VariantEventStoreAttributes,
}

struct EnumEventStoreAttributes {
    /// Event namespace like `accounts` or `organisations`
    event_namespace: String,

    /// Entity type like `user` or `organisation`
    entity_type: String,
}

struct EnumExt<'a> {
    ident: Ident,
    enum_body: &'a DataEnum,
    event_store_attributes: EnumEventStoreAttributes,
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

/// Get attributes as a nice struct from something like
// `#[event_store(event_namespace = "store", event_type = "ThingCreated", entity_type = "thing")]`
fn get_variant_event_attributes(variant: &Variant) -> Result<VariantExt, String> {
    // TODO: Validate there's only one event_store attr
    attributes_map(&variant.attrs)
        .and_then(|mut keys_values| {
            let out = VariantEventStoreAttributes {
                event_type: keys_values
                    .remove(&String::from("event_type"))
                    .ok_or(format!(
                        "Failed to find event_type property on {}",
                        variant.ident
                    ))?,
                event_namespace: keys_values.remove(&String::from("event_namespace")),
                entity_type: keys_values.remove(&String::from("entity_type")),
            };

            Ok(out)
        })
        .map(|event_store_attributes| VariantExt {
            variant,
            event_store_attributes,
        })
}

fn get_enum_event_attributes<'a>(
    parsed: &'a DeriveInput,
    enum_body: &'a DataEnum,
) -> Result<EnumExt<'a>, String> {
    let ident = parsed.ident.clone();
    let event_store_attribute = attributes_map(&parsed.attrs).and_then(|mut keys_values| {
        let attribs = EnumEventStoreAttributes {
            event_namespace: keys_values.remove(&String::from("event_namespace")).ok_or(
                format!(
                    "Failed to find attribute property event_namespace for {}",
                    ident
                ),
            )?,
            entity_type: keys_values
                .remove(&String::from("entity_type"))
                .ok_or(format!(
                    "Failed to find attribute property entity_type for {}",
                    ident
                ))?,
        };

        Ok(attribs)
    })?;

    Ok(EnumExt {
        ident,
        enum_body,
        event_store_attributes: EnumEventStoreAttributes {
            event_namespace: String::new(),
            entity_type: String::new(),
        },
    })
}

// TODO: Different funcs for CreateEvents (enum) and ModifyEvents
// TODO: Function for CreateEvents struct
pub fn derive_create_enum(parsed: &DeriveInput, enum_body: &DataEnum) -> TokenStream {
    // let info = EnumInfo::new(&parsed, &enum_body);
    // let &EnumInfo { ref item_ident, .. } = &info;

    // let ser = impl_serialize(&info);
    // let de = impl_deserialize(&info);

    let item_ident = parsed.ident.clone().into_token_stream();

    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_CREATION_EVENTS_{}", item_ident),
        Span::call_site(),
    );

    let enum_attributes = get_enum_event_attributes(parsed, &enum_body).unwrap();

    // let (impl_generics, ty_generics, _where_clause) = info.generics.split_for_impl();

    let variant_attributes = enum_body
        .variants
        .iter()
        .map(get_variant_event_attributes)
        .collect::<Result<Vec<VariantExt>, String>>()
        .unwrap();

    quote! {
        // #[allow(non_upper_case_globals, unused_attributes, unused_imports)]
        const #dummy_const: () = {
            // extern crate serde;
            // extern crate event_store_derive_internals;

            // use serde::ser;
            // use serde::de::{Deserialize, Deserializer, IntoDeserializer};
            // use serde::ser::{Serialize, Serializer, SerializeMap};

            // impl #impl_generics event_store_derive_internals::Events for #item_ident #ty_generics {}

            // #ser
            // #de
        };
    }
}
