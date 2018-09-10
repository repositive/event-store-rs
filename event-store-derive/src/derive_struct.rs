use ns::StructInfo;
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::{DataStruct, DeriveInput};

fn impl_serialize(info: &StructInfo) -> TokenStream {
    let &StructInfo {
        ref field_idents,
        ref item_ident,
        ref renamed_item_ident_quoted,
        ref renamed_namespace_and_type,
        ref struct_body,
        ref struct_namespace_quoted,
        ..
    } = info;

    let field_idents2 = field_idents.iter();

    let body = struct_body.clone().fields.into_token_stream();

    quote! {
        impl Serialize for #item_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #[derive(Serialize)]
                struct Output #body

                #[derive(Serialize)]
                struct Helper<'a> {
                    #[serde(rename = "type")]
                    event_namespace_and_type: &'a str,
                    event_type: &'a str,
                    event_namespace: &'a str,
                    #[serde(flatten)]
                    payload: &'a Output,
                }

                let out = Helper {
                    event_namespace_and_type: #renamed_namespace_and_type,
                    event_namespace: #struct_namespace_quoted,
                    event_type: #renamed_item_ident_quoted,
                    payload: &Output {
                        #(#field_idents: self.#field_idents2.clone(), )*
                    }
                };

                out.serialize(serializer).map_err(ser::Error::custom)
            }
        }
    }
}

fn impl_deserialize(info: &StructInfo) -> TokenStream {
    let &StructInfo {
        ref field_idents,
        ref item_ident,
        ref renamed_item_ident_quoted,
        ref struct_body,
        ref struct_namespace_quoted,
        ..
    } = info;

    let field_idents2 = field_idents.iter();

    let body = struct_body.clone().fields.into_token_stream();

    quote! {
        impl<'de> Deserialize<'de> for #item_ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                use serde::de;

                #[derive(Deserialize, Clone)]
                struct Output #body

                #[derive(Deserialize, Clone)]
                struct EventIdent {
                    event_type: String,
                    event_namespace: String,
                }

                #[derive(Deserialize, Clone)]
                struct Helper {
                    #[serde(rename = "type")]
                    _event_namespace_and_type: Option<String>,
                    #[serde(flatten)]
                    _event_ident: Option<EventIdent>,
                    #[serde(flatten)]
                    _payload: Output
                }

                let helper = Helper::deserialize(deserializer).map_err(de::Error::custom)?;

                let ident = if let Some(ident) = helper._event_ident {
                    ident
                } else if let Some(ns_and_ty) = helper._event_namespace_and_type {
                    let parts = ns_and_ty.split('.').map(|part| String::from(part)).collect::<Vec<String>>();

                    EventIdent {
                        event_namespace: parts[0].clone(),
                        event_type: parts[1].clone(),
                    }
                } else {
                    return Err(de::Error::custom("No event identifier found"));
                };

                if ident.event_type == #renamed_item_ident_quoted && ident.event_namespace == #struct_namespace_quoted {
                    Ok(#item_ident {
                        #(#field_idents: helper._payload.#field_idents2,)*
                    })
                } else {
                    Err(de::Error::custom("Incorrect event identifier"))
                }
            }
        }
    }
}

pub fn derive_struct(parsed: &DeriveInput, struct_body: &DataStruct) -> TokenStream {
    let info = StructInfo::new(&parsed, &struct_body);

    let &StructInfo {
        ref struct_namespace_quoted,
        ref item_ident,
        ref renamed_namespace_and_type,
        ref renamed_item_ident_quoted,
        ..
    } = &info;

    let ser = impl_serialize(&info);
    let de = impl_deserialize(&info);

    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_STRUCT_FOR_{}", item_ident),
        Span::call_site(),
    );

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_imports)]
        const #dummy_const: () = {
            extern crate serde;
            extern crate event_store_derive_internals;

            use serde::ser;
            use serde::de::{Deserialize, Deserializer};
            use serde::ser::{Serialize, Serializer, SerializeMap};

            impl event_store_derive_internals::EventData for #item_ident {
                fn event_namespace_and_type() -> &'static str { #renamed_namespace_and_type }
                fn event_namespace() -> &'static str { #struct_namespace_quoted }
                fn event_type() -> &'static str { #renamed_item_ident_quoted }
            }

            #ser
            #de
        };
    }
}
