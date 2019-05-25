use proc_macro2::Ident;
use syn::DataEnum;

/// Attributes taken from `#[derive()]` statement on an enum variant
#[derive(Default, Debug)]
pub(crate) struct VariantEventStoreAttributes {
    /// Event type like `ThingUpdated`
    pub event_type: String,

    /// Event namespace override from enum definition
    ///
    /// Unused
    pub event_namespace: Option<String>,

    /// Entity override from enum definition
    ///
    /// Unused
    pub entity_type: Option<String>,
}

pub(crate) struct EnumEventStoreAttributes {
    /// Event namespace like `accounts` or `organisations`
    pub event_namespace: String,

    /// Entity type like `user` or `organisation`
    pub entity_type: String,
}

pub(crate) struct EnumExt<'a> {
    pub ident: Ident,
    pub enum_body: &'a DataEnum,
    pub event_store_attributes: EnumEventStoreAttributes,
}
