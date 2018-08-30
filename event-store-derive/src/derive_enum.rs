use ns::get_namespace_from_attributes;
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use std::iter::repeat;
use syn::{DataEnum, DeriveInput};

// TODO: Make args a struct to make passing them around easier
fn impl_serialize(
    item_ident: &TokenStream,
    _struct_body: &DataEnum,
    _ns: &String,
    _ty: &String,
    _variant_idents: &Vec<Ident>,
) -> TokenStream {
    // let variant_idents_rhs = variant_idents.clone();
    // let variant_idents_quoted = variant_idents.iter().map(|ident| ident.to_string());

    // let item_ident_quoted = item_ident.to_string();

    // Number of fields in the struct plust `type`, `event_type` and `event_namespace`
    // let total_fields = variant_idents.len() + 3;

    quote! {
        impl Serialize for #item_ident {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                // serde_json::to_value(&self)
                //     .map_err(ser::Error::custom)
                //     .and_then(|v| {
                //          let mut map = serializer.serialize_map(None)?;
                //             // for (k, v) in self {
                //             //     map.serialize_entry(k, v)?;
                //             // }
                //             map.end().unwrap();

                //             Err(ser::Error::custom("Could not turn item into object"))
                //         // if let serde_json::Value::Object(items) = v {
                //         //     let mut map = serializer.serialize_map(None)?;
                //         //     // for (k, v) in self {
                //         //     //     map.serialize_entry(k, v)?;
                //         //     // }
                //         //     map.end()
                //         // } else {
                //         //     Err(ser::Error::custom("Could not turn item into object"))
                //         // }
                //     })?
                Err(ser::Error::custom("Could not turn item into object"))
            }
        }
    }
}

// TODO: Make args a struct to make passing them around easier
fn impl_deserialize(
    enum_namespace: &Ident,
    item_ident: &TokenStream,
    enum_body: &DataEnum,
    _ns: &String,
    _ty: &String,
    // TODO: Make a struct that holds variant metadata
    variant_idents: &Vec<Ident>,
) -> TokenStream {
    let item_idents = repeat(item_ident);

    let body = enum_body.clone().variants.into_token_stream();

    let variant_namespaces_quoted = enum_body.variants.iter().map(|variant| {
        get_namespace_from_attributes(&variant.attrs)
            .unwrap_or(enum_namespace.clone())
            .to_string()
    });

    let variant_types_quoted = enum_body
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
                    pub event_type_and_namespace: Option<String>,
                    pub event_type: Option<String>,
                    pub event_namespace: Option<String>,
                }

                #[derive(Deserialize, Debug)]
                enum Output {
                    #body
                };

                // TODO: Remove reliance on serde_json and make this generic
                let v = serde_json::Value::deserialize(deserializer).map_err(de::Error::custom)?;

                let mut type_helper = Helper::deserialize(&v).map_err(de::Error::custom)?;

                // Map old-style event to new-style if new-style is not defined
                if let Some(ref ns_and_ty) = &type_helper.event_type_and_namespace {
                    if type_helper.event_type.is_none() && type_helper.event_namespace.is_none() {
                        let parts: Vec<String> = ns_and_ty.clone().split('.').map(|part| String::from(part)).collect();

                        type_helper.event_namespace = Some(parts[0].clone());
                        type_helper.event_type = Some(parts[1].clone());
                    }
                }

                // TODO: Deserialize event where `Type` (in struct) has different name to enum variant. Does this make sense to do?
                match (&type_helper.event_namespace, &type_helper.event_type) {
                    (Some(ref ns), Some(ref ty)) => {
                        match (ns.as_str(), ty.as_str()) {
                            #((#variant_namespaces_quoted, #variant_types_quoted) => {
                                let variant_value = serde_json::from_value(v).map_err(de::Error::custom)?;
                                Ok(#item_idents::#variant_idents(variant_value))
                            },)*
                            _ => Err(de::Error::custom("Could not find matching variant"))
                        }
                    },
                    _ => Err(de::Error::custom("Could not deserialize event"))
                }
            }
        }
    }
}

pub fn derive_enum(parsed: &DeriveInput, enum_body: &DataEnum) -> TokenStream {
    let default_namespace = get_namespace_from_attributes(&parsed.attrs)
        .expect("Namespace attribute must be provided at the enum level");

    let item_ident = parsed.clone().ident.into_token_stream();
    // let item_idents = repeat(&item_ident);

    // let variant_idents = enum_body.variants.iter().map(|v| v.ident.clone());
    // let variant_namespaces = enum_body.variants.iter().map(|variant| {
    //     get_namespace_from_attributes(&variant.attrs).unwrap_or(default_namespace.clone())
    // });

    // let namespaced_variants_quoted = enum_body
    //     .variants
    //     .iter()
    //     .map(|v| v.ident.clone())
    //     .zip(enum_body.variants.iter())
    //     .map(|(ns, variant)| {
    //         TokenStream::from_str(&format!("\"{}.{}\"", ns, variant.ident)).expect("Variant name")
    //     }).collect::<Vec<TokenStream>>();

    let collected = enum_body
        .variants
        .iter()
        .map(|v| v.ident.clone())
        .collect::<Vec<Ident>>();

    let ser = impl_serialize(
        &item_ident,
        &enum_body,
        &String::new(),
        &String::new(),
        &collected,
    );
    let de = impl_deserialize(
        &default_namespace,
        &item_ident,
        &enum_body,
        &String::new(),
        &String::new(),
        &collected,
    );

    let out = quote!{
        impl event_store_derive_internals::EventData for #item_ident {
            fn event_namespace_and_type(&self) -> &'static str {
                // match self {
                //     #(#item_idents::#variant_idents(_) => #namespaced_variants_quoted,)*
                // }
                "TODO"
            }

            fn event_namespace(&self) -> &'static str {
                // TODO
                "TODO"
            }

            fn event_type(&self) -> &'static str {
                // TODO
                "TODO"
            }
        }

        #ser

        #de
    };

    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_ENUM_FOR_{}", item_ident),
        Span::call_site(),
    );

    quote! {
        #[allow(non_upper_case_globals)]
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
