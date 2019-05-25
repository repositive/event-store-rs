use crate::attributes_map;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{DataStruct, DeriveInput, Fields};

struct StructExt<'a> {
    event_namespace: String,
    entity_type: String,
    ident: Ident,
    body: &'a DataStruct,
}

fn impl_deserialize(struct_ext: &StructExt) -> Result<TokenStream, String> {
    // let ident = &enum_attributes.ident;
    // let idents = repeat(ident);

    // let variant_idents = enum_attributes
    //     .enum_body
    //     .variants
    //     .iter()
    //     .map(|variant| variant.ident.clone());

    // let variant_idents2 = variant_idents.clone();
    // let variant_idents3 = variant_idents.clone();

    // let variant_types = enum_attributes
    //     .enum_body
    //     .variants
    //     .iter()
    //     .map(|variant| variant.fields.clone());

    // let ns = enum_attributes
    //     .event_store_attributes
    //     .event_namespace
    //     .clone();
    // let entity_ty = enum_attributes.event_store_attributes.entity_type.clone();

    let ident = struct_ext.ident.clone();
    let ident_str = ident.to_string();
    let ns = struct_ext.event_namespace.clone();
    let entity_ty = struct_ext.entity_type.clone();

    let fields = if let Fields::Named(fields) = struct_ext.body.fields.clone() {
        Ok(fields.named)
    } else {
        Err(String::from("Unnamed and unit structs are not supported"))
    }?;

    let field_idents = fields
        .iter()
        .cloned()
        .map(|field| field.ident.ok_or(format!("Could not get field identifier")))
        .collect::<Result<Vec<Ident>, String>>()?;
    let field_idents_2 = field_idents.clone();

    let body = fields.into_token_stream();

    Ok(quote! {
        impl<'de> serde::de::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                use serde::de::Error;


                #[derive(serde_derive::Deserialize, Debug)]
                struct Helper {
                    event_namespace: String,
                    event_type: String,
                    entity_type: String,
                    #body
                }

                Helper::deserialize(deserializer).and_then(|helper| {
                    if helper.event_type != #ident_str {
                        Err(serde::de::Error::custom(format!("expected event_type {}, got {}", #ns, helper.event_namespace)))
                    } else if helper.event_namespace != #ns {
                        Err(serde::de::Error::custom(format!("expected event_namespace {}, got {}", #ns, helper.event_namespace)))
                    } else if helper.entity_type != #entity_ty {
                        Err(serde::de::Error::custom(format!("expected entity_type {}, got {}", #entity_ty, helper.entity_type)))
                    } else {
                        Ok(#ident {
                            #(#field_idents: helper.#field_idents_2,)*
                        })
                    }
                })
            }
        }
    })
}

pub fn derive_create_struct(parsed: &DeriveInput, struct_body: &DataStruct) -> TokenStream {
    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_CREATION_EVENTS_{}", parsed.ident),
        Span::call_site(),
    );

    let struct_ext = attributes_map(&parsed.attrs)
        .and_then(|mut keys_values| {
            let attribs = StructExt {
                body: struct_body,
                ident: parsed.ident.clone(),
                event_namespace: keys_values.remove(&String::from("event_namespace")).ok_or(
                    format!(
                        "Failed to find attribute property event_namespace for {}",
                        parsed.ident
                    ),
                )?,
                entity_type: keys_values
                    .remove(&String::from("entity_type"))
                    .ok_or(format!(
                        "Failed to find attribute property entity_type for {}",
                        parsed.ident
                    ))?,
            };

            Ok(attribs)
        })
        .unwrap();

    // let enum_attributes = get_enum_event_attributes(parsed, &enum_body).unwrap();

    // // let variant_attributes = enum_body
    // //     .variants
    // //     .iter()
    // //     .map(get_variant_event_attributes)
    // //     .collect::<Result<Vec<VariantExt>, String>>()
    // //     .unwrap();

    let de = impl_deserialize(&struct_ext).unwrap();

    quote! {
        // #[allow(non_upper_case_globals, unused_attributes, unused_imports)]
        const #dummy_const: () = {
            #de
        };
    }
}
