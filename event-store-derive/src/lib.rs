#![recursion_limit = "128"]

extern crate proc_macro;

mod derive_enum;
mod enum_helpers;

use crate::derive_enum::derive_enum;
use proc_macro2::{Ident, Span};
use syn::{Data, DeriveInput};

/// Name of attribute used in `#[derive()]` statements
const PROC_MACRO_NAME: &'static str = "event_store";

#[proc_macro_derive(CreateEvents, attributes(event_store))]
pub fn derive_create_events(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    match input.data {
        Data::Enum(ref body) => derive_enum(
            &input,
            &body,
            Ident::new("EventStoreCreateEvents", Span::call_site()),
        ),
        _ => panic!("Entity create events must be an enum"),
    }
    .into()
}

#[proc_macro_derive(UpdateEvents, attributes(event_store))]
pub fn derive_update_events(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    match input.data {
        Data::Enum(ref body) => derive_enum(
            &input,
            &body,
            Ident::new("EventStoreUpdateEvents", Span::call_site()),
        ),
        _ => panic!("Entity update events must be an enum"),
    }
    .into()
}
