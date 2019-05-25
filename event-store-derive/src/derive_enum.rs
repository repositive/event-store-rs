use crate::enum_helpers::{attributes_map, EnumEventStoreAttributes, EnumExt};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::iter::repeat;
use syn::{DataEnum, DeriveInput};

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

fn impl_serialize(enum_attributes: &EnumExt) -> Result<TokenStream, String> {
    let ident = &enum_attributes.ident;
    let idents = repeat(ident);

    let variant_idents = enum_attributes
        .enum_body
        .variants
        .iter()
        .map(|variant| variant.ident.clone());

    Ok(quote! {
        impl Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                match self {
                    #(#idents::#variant_idents(evt) =>
                        evt.serialize(serializer)
                    ,)*
                }
            }
        }
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

                #[derive(serde_derive::Deserialize)]
                #[serde(tag = "event_type")]
                enum HelperVariants {
                    #(#variant_idents(#variant_types),)*
                }

                #[derive(serde_derive::Deserialize)]
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

pub fn derive_enum(parsed: &DeriveInput, enum_body: &DataEnum, trait_bound: Ident) -> TokenStream {
    let item_ident = parsed.ident.clone().into_token_stream();

    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_CREATION_EVENTS_{}", item_ident),
        Span::call_site(),
    );

    let enum_attributes = get_enum_event_attributes(parsed, &enum_body).unwrap();

    let ser = impl_serialize(&enum_attributes).unwrap();
    let de = impl_deserialize(&enum_attributes).unwrap();

    quote! {
        const #dummy_const: () = {
            extern crate serde;
            extern crate event_store_derive_internals;

            use serde::ser;
            use serde::de::{Deserialize, Deserializer, IntoDeserializer};
            use serde::ser::{Serialize, Serializer, SerializeMap};

            impl event_store_derive_internals::#trait_bound for #item_ident {}

            #ser
            #de
        };
    }
}
