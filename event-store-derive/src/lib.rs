extern crate proc_macro;

mod derive_enum;

use crate::derive_enum::derive_enum;
use syn::Data;
use syn::DeriveInput;

/// Name of attribute used in `#[derive()]` statements
const PROC_MACRO_NAME: &'static str = "event_store";

fn expand_derive_namespace(parsed: &DeriveInput) -> proc_macro2::TokenStream {
    match parsed.data {
        Data::Enum(ref body) => derive_enum(&parsed, &body),
        _ => panic!("Namespace can only be derived on enums"),
    }
}

#[proc_macro_derive(Events, attributes(event_store, serde))]
pub fn derive_events(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    expand_derive_namespace(&input).into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
