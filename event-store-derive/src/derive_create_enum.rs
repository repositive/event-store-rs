use crate::attributes_map;
use crate::enum_helpers::{
    EnumEventStoreAttributes, EnumExt, VariantEventStoreAttributes, VariantExt,
};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::iter::repeat;
use syn::{DataEnum, DeriveInput, Variant};

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
    let event_store_attributes = attributes_map(&parsed.attrs).and_then(|mut keys_values| {
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
        event_store_attributes,
    })
}

fn impl_deserialize(enum_attributes: &EnumExt) -> Result<TokenStream, String> {
    let ident = &enum_attributes.ident;
    let idents = repeat(ident);

    let variant_idents = enum_attributes
        .enum_body
        .variants
        .iter()
        .map(|variant| variant.ident.clone());

    let variant_idents2 = variant_idents.clone();
    let variant_idents3 = variant_idents.clone();

    let variant_types = enum_attributes
        .enum_body
        .variants
        .iter()
        .map(|variant| variant.fields.clone());

    let ns = enum_attributes
        .event_store_attributes
        .event_namespace
        .clone();
    let entity_ty = enum_attributes.event_store_attributes.entity_type.clone();

    Ok(quote! {
        impl<'de> serde::de::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                use serde::de::Error;

                #[derive(serde_derive::Deserialize, Debug)]
                #[serde(tag = "event_type")]
                enum HelperVariants {
                    #(#variant_idents(#variant_types),)*
                }

                #[derive(serde_derive::Deserialize, Debug)]
                struct Helper {
                    event_namespace: String,
                    entity_type: String,
                    #[serde(flatten)]
                    variants: HelperVariants
                }

                Helper::deserialize(deserializer).and_then(|helper| {
                    if helper.event_namespace != #ns {
                        Err(serde::de::Error::custom(format!("expected event_namespace {}, got {}", #ns, helper.event_namespace)))
                    } else if helper.entity_type != #entity_ty {
                        Err(serde::de::Error::custom(format!("expected entity_type {}, got {}", #entity_ty, helper.entity_type)))
                    } else {
                        Ok(match helper.variants {
                            #(HelperVariants::#variant_idents2(evt) => #idents::#variant_idents3(evt),)*
                        })
                    }
                })
            }
        }
    })
}

pub fn derive_create_enum(parsed: &DeriveInput, enum_body: &DataEnum) -> TokenStream {
    let item_ident = parsed.ident.clone().into_token_stream();

    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_CREATION_EVENTS_{}", item_ident),
        Span::call_site(),
    );

    let enum_attributes = get_enum_event_attributes(parsed, &enum_body).unwrap();

    // let variant_attributes = enum_body
    //     .variants
    //     .iter()
    //     .map(get_variant_event_attributes)
    //     .collect::<Result<Vec<VariantExt>, String>>()
    //     .unwrap();

    let de = impl_deserialize(&enum_attributes).unwrap();

    quote! {
        // #[allow(non_upper_case_globals, unused_attributes, unused_imports)]
        const #dummy_const: () = {
            extern crate serde;
            extern crate event_store_derive_internals;

            use serde::ser;
            use serde::de::{Deserialize, Deserializer, IntoDeserializer};
            use serde::ser::{Serialize, Serializer, SerializeMap};

            // impl #impl_generics event_store_derive_internals::Events for #item_ident #ty_generics {}

            // #ser
            #de
        };
    }
}
