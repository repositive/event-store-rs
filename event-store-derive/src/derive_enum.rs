use ns::get_namespace_from_attributes;
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use std::iter::repeat;
use syn::{DataEnum, DeriveInput};

// TODO: Make args a struct to make passing them around easier
fn impl_serialize(
    item_ident: &TokenStream,
    enum_body: &DataEnum,
    _ns: &String,
    _ty: &String,
    variant_idents: &Vec<Ident>,
) -> TokenStream {
    // let item_ident_quoted = item_ident.to_string();

    let item_idents = repeat(item_ident);

    // TODO: Move into own function
    let struct_idents = enum_body
        .variants
        .iter()
        .map(|variant| {
            variant
                .fields
                .iter()
                .next()
                .map(|field| field.ty.clone().into_token_stream())
                .expect("Expected struct type")
        }).collect::<Vec<TokenStream>>();

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

                let (payload, ns_and_ty, ns, ty) = match self {
                    #(#item_idents::#variant_idents(evt) => {
                        let p: serde_json::Value = serde_json::to_value(evt).expect("Ser");

                        (
                            p,
                            #struct_idents::event_namespace_and_type().into(),
                            #struct_idents2::event_namespace().into(),
                            #struct_idents3::event_type().into()
                        )
                    },)*
                };

                let out = Output {
                    payload,
                    event_type_and_namespace: ns_and_ty,
                    event_namespace: ns,
                    event_type: ty,
                };

                out.serialize(serializer).map_err(ser::Error::custom)
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
    // let item_idents = repeat(item_ident.clone());
    // let item_idents2 = repeat(item_ident.clone());
    // let item_idents3 = repeat(item_ident.clone());

    let namespaces = enum_body
        .variants
        .iter()
        .map(|variant| {
            get_namespace_from_attributes(&variant.attrs).unwrap_or(default_namespace.clone())
        }).collect::<Vec<Ident>>();

    let namespaces_quoted = namespaces
        .iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<String>>();

    let variant_idents = enum_body
        .variants
        .iter()
        .map(|v| v.ident.clone())
        .collect::<Vec<Ident>>();

    // let variant_idents2 = variant_idents.clone();
    // let variant_idents3 = variant_idents.clone();

    let types_quoted = variant_idents
        .iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<String>>();

    let namespace_and_types_quoted = namespaces_quoted
        .iter()
        .zip(variant_idents.iter())
        .map(|(ns, ty)| format!("{}.{}", ns, ty))
        .collect::<Vec<String>>();

    let ser = impl_serialize(
        &item_ident,
        &enum_body,
        &String::new(),
        &String::new(),
        &variant_idents,
    );
    let de = impl_deserialize(
        &default_namespace,
        &item_ident,
        &enum_body,
        &String::new(),
        &String::new(),
        &variant_idents,
    );

    // TODO: Move into own function
    let struct_idents = enum_body
        .variants
        .iter()
        .map(|variant| {
            variant
                .fields
                .iter()
                .next()
                .map(|field| field.ty.clone().into_token_stream())
                .expect("Expected struct type")
        }).collect::<Vec<TokenStream>>();

    let out = quote!{
        // Get the type or namespace of an instance of an events enum
        impl event_store_derive_internals::Events for #item_ident { }

        #(
            impl event_store_derive_internals::EventData for #struct_idents {
                fn event_namespace_and_type() -> &'static str {
                    #namespace_and_types_quoted
                }

                fn event_namespace() -> &'static str {
                    #namespaces_quoted
                }

                fn event_type() -> &'static str {
                    #types_quoted
                }
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
