#![recursion_limit = "256"]

#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use proc_macro::TokenStream;
use syn::DeriveInput;

mod derive_enum;
mod derive_struct;
mod ns;

const PROC_MACRO_NAME: &'static str = "event_store";

#[proc_macro_derive(Events, attributes(event_store))]
pub fn derive_events(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    ns::expand_derive_namespace(&input).into()
}

#[proc_macro_derive(EventData, attributes(event_store))]
pub fn derive_eventdata(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    ns::expand_derive_namespace(&input).into()
}

// // TODO: Use this by returning Result<>s from derive funcs
// fn compile_error(message: String) -> proc_macro2::TokenStream {
//     quote! {
//         compile_error!(#message);
//     }
// }
