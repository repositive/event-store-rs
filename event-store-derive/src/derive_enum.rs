use ns::get_enum_struct_names;
use ns::get_quoted_namespaces;
use ns::EnumInfo;
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use std::iter::repeat;
use syn::{DataEnum, DeriveInput};

fn impl_serialize(info: &EnumInfo) -> TokenStream {
    let EnumInfo {
        item_ident,
        variant_idents,
        ..
    } = info;

    let item_idents = repeat(item_ident);

    let struct_idents = get_enum_struct_names(&info.enum_body);

    let struct_idents2 = struct_idents.clone();
    let struct_idents3 = struct_idents.clone();

    quote! {
        impl Serialize for #item_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                use serde_json;
                use event_store_derive_internals::EventData;

                #[derive(Serialize)]
                struct Output {
                    #[serde(rename = "type")]
                    event_type_and_namespace: String,
                    event_type: String,
                    event_namespace: String,
                    #[serde(flatten)]
                    payload: serde_json::Value,
                }

                // TODO: Handle rename attribs on enum variant

                let out = match self {
                    #(#item_idents::#variant_idents(evt) => {
                        let payload: serde_json::Value = serde_json::to_value(evt).expect("Ser");

                        Output {
                            payload,
                            event_type_and_namespace: #struct_idents::event_namespace_and_type().into(),
                            event_namespace: #struct_idents2::event_namespace().into(),
                            event_type: #struct_idents3::event_type().into()
                        }
                    },)*
                };

                out.serialize(serializer).map_err(ser::Error::custom)
            }
        }
    }
}

fn impl_deserialize(info: &EnumInfo) -> TokenStream {
    let EnumInfo {
        item_ident,
        variant_idents,
        ..
    } = info;

    let item_idents = repeat(&info.item_ident);
    let body = info.enum_body.clone().variants.into_token_stream();
    let variant_namespaces_quoted = get_quoted_namespaces(&info.enum_body, &info.enum_namespace);

    let variant_types_quoted = info
        .enum_body
        .variants
        .iter()
        .map(|variant| variant.ident.clone().to_string());

    quote! {
        impl<'de> Deserialize<'de> for #item_ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                use serde::de;

                #[derive(Deserialize, Debug)]
                struct Helper {
                    #[serde(rename = "type")]
                    event_type_and_namespace: Option<String>,
                    event_type: Option<String>,
                    event_namespace: Option<String>,
                    #[serde(flatten)]
                    // TODO: Remove reliance on serde_json and make this generic
                    payload: serde_json::Value,
                }

                #[derive(Deserialize, Debug)]
                enum Output {
                    #body
                };

                let type_helper = Helper::deserialize(deserializer).map_err(de::Error::custom)?;

                let (ns, ty) = match type_helper {
                    Helper { event_type: Some(ty), event_namespace: Some(ns), ..  } => {
                        (ns, ty)
                    },
                    // Map old-style event to new-style if new-style is not defined
                    Helper { event_type_and_namespace: Some(ns_and_ty), ..  } => {
                        let parts: Vec<String> = ns_and_ty
                            .split('.')
                            .map(|part| String::from(part))
                            .collect();

                        (parts[0].clone(), parts[1].clone())
                    },
                    _ => return Err(de::Error::custom("Event type and namespace not given"))
                };

                match (ns.as_str(), ty.as_str()) {
                    #((#variant_namespaces_quoted, #variant_types_quoted) => {
                        let variant_value = serde_json::from_value(type_helper.payload)
                            .map_err(de::Error::custom)?;

                        Ok(#item_idents::#variant_idents(variant_value))
                    },)*
                    _ => Err(de::Error::custom("Could not find matching variant"))
                }
            }
        }
    }
}

pub fn derive_enum(parsed: &DeriveInput, enum_body: &DataEnum) -> TokenStream {
    let info = EnumInfo::new(&parsed, &enum_body);
    let &EnumInfo {
        ref enum_namespace,
        ref enum_body,
        ref item_ident,
        ref variant_idents,
        ..
    } = &info;

    let namespaces_quoted = get_quoted_namespaces(&enum_body, &enum_namespace);

    let types_quoted = variant_idents.iter().map(|ident| ident.to_string());

    let namespace_and_types_quoted = namespaces_quoted
        .iter()
        .zip(variant_idents.iter())
        .map(|(ns, ty)| format!("{}.{}", ns, ty))
        .collect::<Vec<String>>();

    let ser = impl_serialize(&info);
    let de = impl_deserialize(&info);

    let struct_idents = get_enum_struct_names(&enum_body);

    let out = quote!{
        // Get the type or namespace of an instance of an events enum
        impl event_store_derive_internals::Events for #item_ident { }

        #(
            impl event_store_derive_internals::EventData for #struct_idents {
                fn event_namespace_and_type() -> &'static str { #namespace_and_types_quoted }
                fn event_namespace() -> &'static str { #namespaces_quoted }
                fn event_type() -> &'static str { #types_quoted }
            }
        )*
        #ser
        #de
    };

    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_ENUM_FOR_{}", item_ident),
        Span::call_site(),
    );

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_imports)]
        const #dummy_const: () = {
            extern crate serde;
            extern crate serde_json;
            extern crate event_store_derive_internals;

            use serde::ser;
            use serde::de::{Deserialize, Deserializer};
            use serde::ser::{Serialize, Serializer, SerializeMap};

            #out
        };
    }
}
