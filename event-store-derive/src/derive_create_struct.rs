use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{DataStruct, DeriveInput};

pub fn derive_create_struct(parsed: &DeriveInput, struct_body: &DataStruct) -> TokenStream {
    let item_ident = parsed.ident.clone().into_token_stream();

    let dummy_const = Ident::new(
        &format!("_IMPL_EVENT_STORE_CREATION_EVENTS_{}", item_ident),
        Span::call_site(),
    );

    // let enum_attributes = get_enum_event_attributes(parsed, &enum_body).unwrap();

    // // let variant_attributes = enum_body
    // //     .variants
    // //     .iter()
    // //     .map(get_variant_event_attributes)
    // //     .collect::<Result<Vec<VariantExt>, String>>()
    // //     .unwrap();

    // let de = impl_deserialize(&enum_attributes).unwrap();

    quote! {
        // #[allow(non_upper_case_globals, unused_attributes, unused_imports)]
        const #dummy_const: () = {

        };
    }
}
