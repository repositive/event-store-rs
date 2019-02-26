use crate::ns::StructInfo;
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::Fields;
use syn::{token, DataStruct, DeriveInput, GenericParam, Lifetime};

fn impl_serialize(info: &StructInfo) -> TokenStream {
    let &StructInfo {
        ref field_idents,
        ref item_ident,
        ref renamed_item_ident_quoted,
        ref renamed_namespace_and_type,
        ref struct_body,
        ref struct_namespace_quoted,
        ref generics,
        ..
    } = info;

    let field_idents2 = field_idents.iter();

    let body = if let Fields::Named(fields) = struct_body.clone().fields {
        fields.named.into_token_stream()
    } else {
        panic!("Unnamed and unit structs are not supported");
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics Serialize for #item_ident #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #[derive(Serialize)]
                struct Helper #ty_generics #where_clause {
                    #[serde(rename = "type")]
                    event_namespace_and_type: &'static str,
                    event_type: &'static str,
                    event_namespace: &'static str,
                    #body
                }

                let out = Helper {
                    event_namespace_and_type: #renamed_namespace_and_type,
                    event_namespace: #struct_namespace_quoted,
                    event_type: #renamed_item_ident_quoted,
                    #(#field_idents: self.#field_idents2.clone(), )*
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
        ref generics,
        ..
    } = info;

    let field_idents2 = field_idents.iter();

    let body = if let Fields::Named(fields) = struct_body.clone().fields {
        fields.named.into_token_stream()
    } else {
        panic!("Unnamed and unit structs are not supported");
    };

    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // let lifetimes = if generics.params.is_empty() {
    //     // quote!('de)
    //     generics.params.clone().into_token_stream()
    // } else {
    //     generics.params.clone().into_token_stream()
    // };

    let mut field_list = field_idents
        .clone()
        .iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<String>>();

    // field_list.append(&mut vec![
    //     "event_type".to_string(),
    //     "event_namespace".to_string(),
    //     "type".to_string(),
    // ]);

    // println!("FIELDS {} {:?}", item_ident, field_list);

    let lifetimes = generics.params.clone();

    quote! {
        impl< 'de, #lifetimes > serde::Deserialize<'de> for #item_ident #ty_generics #where_clause {
            fn deserialize<__D>(deserializer: __D) -> serde::export::Result<Self, __D::Error>
            where
                __D: serde::Deserializer<'de>,
            {
                use serde::de;

                // #[derive(Deserialize)]
                // struct Legacy<'l> {
                //     #[serde(rename = "type")]
                //     event_namespace_and_type: &'l str,
                // }

                // #[derive(Deserialize)]
                // struct Split<'s> {
                //     event_namespace: &'s str,
                //     event_type: &'s str,
                // }

                // #[derive(Deserialize)]
                // enum EventIdent<'ei> {
                //     Legacy(Legacy<'ei>),
                //     Split(Split<'ei>)
                // }

                // let event_ident = EventIdent::deserialize(deserializer)?;

                // let ident: Split = match event_ident {
                //     EventIdent::Legacy(l) => {
                //         let parts = l.event_namespace_and_type.split('.');

                //         Split {
                //             event_namespace: parts.next().unwrap(),
                //             event_type: parts.next().unwrap(),
                //         }
                //     },
                //     EventIdent::Split(s) => s
                // };

                // if ident.event_type == #renamed_item_ident_quoted && ident.event_namespace == #struct_namespace_quoted {
                //     // Ok(#item_ident {
                //     //     #(#field_idents: helper.#field_idents2,)*
                //     // })
                //     Err(de::Error::custom("Not implemented!"))
                // } else {
                //     Err(de::Error::custom("Incorrect event identifier"))
                // }

                // ----

                #[derive(Deserialize, Clone)]
                struct EventIdent {
                    event_type: String,
                    event_namespace: String,
                }

                #[derive(Deserialize, Clone)]
                struct Helper #ty_generics {
                    #[serde(rename = "type")]
                    _event_namespace_and_type: Option<String>,
                    #[serde(flatten)]
                    _event_ident: Option<EventIdent>,
                    #body
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
                        #(#field_idents: helper.#field_idents2,)*
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
        ref generics,
        ..
    } = &info;

    let ser = impl_serialize(&info);
    let de = impl_deserialize(&info);

    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_STRUCT_FOR_{}", item_ident),
        Span::call_site(),
    );

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_imports)]
        const #dummy_const: () = {
            extern crate serde;
            extern crate serde_derive;
            extern crate event_store_derive_internals;

            use serde::ser;
            use serde::de::{Deserialize, Deserializer};
            use serde::ser::{Serialize, Serializer, SerializeMap};

            impl #impl_generics event_store_derive_internals::EventData for #item_ident #ty_generics #where_clause {
                fn event_namespace_and_type() -> &'static str { #renamed_namespace_and_type }
                fn event_namespace() -> &'static str { #struct_namespace_quoted }
                fn event_type() -> &'static str { #renamed_item_ident_quoted }
            }

            #ser
            #de
        };
    }
}
