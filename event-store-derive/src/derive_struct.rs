use ns::get_namespace_from_attributes;
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::{DataStruct, DeriveInput};

// TODO: Make args a struct to make passing them around easier
fn impl_deserialize(
    item_ident: &TokenStream,
    struct_body: &DataStruct,
    ns: &String,
    ty: &String,
    field_idents: &Vec<Ident>,
) -> TokenStream {
    let body = struct_body.clone().fields.into_token_stream();

    let field_idents_rhs = field_idents.clone();

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
                struct Output #body;

                impl From<Output> for #item_ident {
                    fn from(out: Output) -> #item_ident {
                        #item_ident {
                            #(#field_idents: out.#field_idents_rhs,)*
                        }
                    }
                }

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
                        if ns != #ns || ty != #ty {
                            Err(de::Error::custom("Data does not match types"))
                        } else {
                            let out = Output::deserialize(v).map_err(de::Error::custom)?;

                            Ok(out.into())
                        }
                    },
                    _ => Err(de::Error::custom("Could not deserialize event"))
                }
            }
        }
    }
}

// TODO: Make args a struct to make passing them around easier
fn impl_serialize(
    item_ident: &TokenStream,
    _struct_body: &DataStruct,
    _ns: &String,
    _ty: &String,
    field_idents: &Vec<Ident>,
) -> TokenStream {
    let field_idents_rhs = field_idents.clone();
    let field_idents_quoted = field_idents.iter().map(|ident| ident.to_string());

    let item_ident_quoted = item_ident.to_string();

    // Number of fields in the struct plust `type`, `event_type` and `event_namespace`
    let total_fields = field_idents.len() + 3;

    quote! {
        impl Serialize for #item_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut state = serializer.serialize_struct(#item_ident_quoted, #total_fields)?;

                #(state.serialize_field(#field_idents_quoted, &self.#field_idents_rhs)?;)*

                state.end()
            }
        }
    }
}

pub fn derive_struct(parsed: &DeriveInput, struct_body: &DataStruct) -> TokenStream {
    let struct_namespace = get_namespace_from_attributes(&parsed.attrs)
        .expect("Namespace attribute must be provided at the struct level");

    let item_ident = parsed.clone().ident.into_token_stream();

    let namespaced_ident = format!("\"{}.{}\"", struct_namespace, item_ident);

    // Turn tokens into string literals so comparisons can be made
    let ns = format!("{}", struct_namespace);
    let ty = format!("{}", item_ident);

    let field_idents = struct_body
        .fields
        .iter()
        .filter_map(|field| field.clone().ident)
        .collect::<Vec<Ident>>();

    let ser = impl_serialize(&item_ident, &struct_body, &ns, &ty, &field_idents);
    let de = impl_deserialize(&item_ident, &struct_body, &ns, &ty, &field_idents);

    // TODO: Split out into different funcs that return `Fragment`s
    let out = quote!{
        impl event_store_derive_internals::EventData for #item_ident {
            fn namespaced_type(&self) -> &'static str {
                #namespaced_ident
            }
        }

        #ser

        #de
    };

    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_STRUCT_FOR_{}", item_ident),
        Span::call_site(),
    );

    quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate serde;
            extern crate serde_json;
            extern crate event_store_derive_internals;

            use serde::de::{Deserialize, Deserializer};
            use serde::ser::{Serialize, Serializer, SerializeStruct};

            #out
        };
    }
}