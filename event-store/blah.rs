#![feature(prelude_import)]
#![no_std]
//! Event store

#![deny(missing_docs)]
// enable the await! macro, async support, and the new std::Futures api.
#![feature(await_macro, async_await, futures_api)]
// only needed to manually implement a std future:
#![feature(arbitrary_self_types)]
#[prelude_import]
use ::std::prelude::v1::*;
#[macro_use]
extern crate std as std;

#[macro_use]
extern crate serde_derive;

mod aggregator {


    //! Aggregator trait
    use crate::store_query::StoreQuery;
    use event_store_derive_internals::Events;
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;
    /// Aggregator trait
    pub trait Aggregator<E: Events, A: Clone, Q: StoreQuery>: Clone + Debug +
     Default + PartialEq + Serialize + for<'de> Deserialize<'de> {
        /// Apply an event `E` to `acc`, returning a copy of `Self` with updated fields. Can also just
        /// return `acc` if nothing has changed.
        fn apply_event(acc: Self, event: &E)
        -> Self;
        /// Produce a query object from some query arguments
        fn query(query_args: A)
        -> Q;
    }
}
mod event {
    use crate::event_context::EventContext;
    use chrono::prelude::*;
    use event_store_derive_internals::EventData;
    use serde_derive::{Deserialize, Serialize};
    use uuid::Uuid;
    /// Event with `EventData`, `EventContext` and a `Uuid` ID
    ///
    /// This is what gets stored in the store and emitted from the emitter
    pub struct Event<D> {
        /// Event data payload
        pub data: D,
        /// Event context
        pub context: EventContext,
        /// Event UUID
        pub id: Uuid,
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _IMPL_SERIALIZE_FOR_Event: () =
        {
            #[allow(unknown_lints)]
            #[allow(rust_2018_idioms)]
            extern crate serde as _serde;
            #[allow(unused_macros)]
            macro_rules! try(( $ __expr : expr ) => {
                             match $ __expr {
                             _serde :: export :: Ok ( __val ) => __val ,
                             _serde :: export :: Err ( __err ) => {
                             return _serde :: export :: Err ( __err ) ; } }
                             });
            #[automatically_derived]
            impl <D> _serde::Serialize for Event<D> where D: _serde::Serialize
             {
                fn serialize<__S>(&self, __serializer: __S)
                 -> _serde::export::Result<__S::Ok, __S::Error> where
                 __S: _serde::Serializer {
                    let mut __serde_state =
                        match _serde::Serializer::serialize_struct(__serializer,
                                                                   "Event",
                                                                   false as
                                                                       usize +
                                                                       1 + 1 +
                                                                       1) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        };
                    match _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                                                        "data",
                                                                        &self.data)
                        {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                                                        "context",
                                                                        &self.context)
                        {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                                                        "id",
                                                                        &self.id)
                        {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _IMPL_DESERIALIZE_FOR_Event: () =
        {
            #[allow(unknown_lints)]
            #[allow(rust_2018_idioms)]
            extern crate serde as _serde;
            #[allow(unused_macros)]
            macro_rules! try(( $ __expr : expr ) => {
                             match $ __expr {
                             _serde :: export :: Ok ( __val ) => __val ,
                             _serde :: export :: Err ( __err ) => {
                             return _serde :: export :: Err ( __err ) ; } }
                             });
            #[automatically_derived]
            impl <'de, D> _serde::Deserialize<'de> for Event<D> where
             D: _serde::Deserialize<'de> {
                fn deserialize<__D>(__deserializer: __D)
                 -> _serde::export::Result<Self, __D::Error> where
                 __D: _serde::Deserializer<'de> {
                    #[allow(non_camel_case_types)]
                    enum __Field { __field0, __field1, __field2, __ignore, }
                    struct __FieldVisitor;
                    impl <'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type
                        Value
                        =
                        __Field;
                        fn expecting(&self,
                                     __formatter:
                                         &mut _serde::export::Formatter)
                         -> _serde::export::fmt::Result {
                            _serde::export::Formatter::write_str(__formatter,
                                                                 "field identifier")
                        }
                        fn visit_u64<__E>(self, __value: u64)
                         -> _serde::export::Result<Self::Value, __E> where
                         __E: _serde::de::Error {
                            match __value {
                                0u64 => _serde::export::Ok(__Field::__field0),
                                1u64 => _serde::export::Ok(__Field::__field1),
                                2u64 => _serde::export::Ok(__Field::__field2),
                                _ =>
                                _serde::export::Err(_serde::de::Error::invalid_value(_serde::de::Unexpected::Unsigned(__value),
                                                                                     &"field index 0 <= i < 3")),
                            }
                        }
                        fn visit_str<__E>(self, __value: &str)
                         -> _serde::export::Result<Self::Value, __E> where
                         __E: _serde::de::Error {
                            match __value {
                                "data" =>
                                _serde::export::Ok(__Field::__field0),
                                "context" =>
                                _serde::export::Ok(__Field::__field1),
                                "id" => _serde::export::Ok(__Field::__field2),
                                _ => { _serde::export::Ok(__Field::__ignore) }
                            }
                        }
                        fn visit_bytes<__E>(self, __value: &[u8])
                         -> _serde::export::Result<Self::Value, __E> where
                         __E: _serde::de::Error {
                            match __value {
                                b"data" =>
                                _serde::export::Ok(__Field::__field0),
                                b"context" =>
                                _serde::export::Ok(__Field::__field1),
                                b"id" =>
                                _serde::export::Ok(__Field::__field2),
                                _ => { _serde::export::Ok(__Field::__ignore) }
                            }
                        }
                    }
                    impl <'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(__deserializer: __D)
                         -> _serde::export::Result<Self, __D::Error> where
                         __D: _serde::Deserializer<'de> {
                            _serde::Deserializer::deserialize_identifier(__deserializer,
                                                                         __FieldVisitor)
                        }
                    }
                    struct __Visitor<'de, D> where
                           D: _serde::Deserialize<'de> {
                        marker: _serde::export::PhantomData<Event<D>>,
                        lifetime: _serde::export::PhantomData<&'de ()>,
                    }
                    impl <'de, D> _serde::de::Visitor<'de> for
                     __Visitor<'de, D> where D: _serde::Deserialize<'de> {
                        type
                        Value
                        =
                        Event<D>;
                        fn expecting(&self,
                                     __formatter:
                                         &mut _serde::export::Formatter)
                         -> _serde::export::fmt::Result {
                            _serde::export::Formatter::write_str(__formatter,
                                                                 "struct Event")
                        }
                        #[inline]
                        fn visit_seq<__A>(self, mut __seq: __A)
                         -> _serde::export::Result<Self::Value, __A::Error>
                         where __A: _serde::de::SeqAccess<'de> {
                            let __field0 =
                                match match _serde::de::SeqAccess::next_element::<D>(&mut __seq)
                                          {
                                          _serde::export::Ok(__val) => __val,
                                          _serde::export::Err(__err) => {
                                              return _serde::export::Err(__err);
                                          }
                                      } {
                                    _serde::export::Some(__value) => __value,
                                    _serde::export::None => {
                                        return _serde::export::Err(_serde::de::Error::invalid_length(0usize,
                                                                                                     &"struct Event with 3 elements"));
                                    }
                                };
                            let __field1 =
                                match match _serde::de::SeqAccess::next_element::<EventContext>(&mut __seq)
                                          {
                                          _serde::export::Ok(__val) => __val,
                                          _serde::export::Err(__err) => {
                                              return _serde::export::Err(__err);
                                          }
                                      } {
                                    _serde::export::Some(__value) => __value,
                                    _serde::export::None => {
                                        return _serde::export::Err(_serde::de::Error::invalid_length(1usize,
                                                                                                     &"struct Event with 3 elements"));
                                    }
                                };
                            let __field2 =
                                match match _serde::de::SeqAccess::next_element::<Uuid>(&mut __seq)
                                          {
                                          _serde::export::Ok(__val) => __val,
                                          _serde::export::Err(__err) => {
                                              return _serde::export::Err(__err);
                                          }
                                      } {
                                    _serde::export::Some(__value) => __value,
                                    _serde::export::None => {
                                        return _serde::export::Err(_serde::de::Error::invalid_length(2usize,
                                                                                                     &"struct Event with 3 elements"));
                                    }
                                };
                            _serde::export::Ok(Event{data: __field0,
                                                     context: __field1,
                                                     id: __field2,})
                        }
                        #[inline]
                        fn visit_map<__A>(self, mut __map: __A)
                         -> _serde::export::Result<Self::Value, __A::Error>
                         where __A: _serde::de::MapAccess<'de> {
                            let mut __field0: _serde::export::Option<D> =
                                _serde::export::None;
                            let mut __field1:
                                    _serde::export::Option<EventContext> =
                                _serde::export::None;
                            let mut __field2: _serde::export::Option<Uuid> =
                                _serde::export::None;
                            while let _serde::export::Some(__key) =
                                      match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                          {
                                          _serde::export::Ok(__val) => __val,
                                          _serde::export::Err(__err) => {
                                              return _serde::export::Err(__err);
                                          }
                                      } {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::export::Option::is_some(&__field0)
                                           {
                                            return _serde::export::Err(<__A::Error
                                                                           as
                                                                           _serde::de::Error>::duplicate_field("data"));
                                        }
                                        __field0 =
                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<D>(&mut __map)
                                                                     {
                                                                     _serde::export::Ok(__val)
                                                                     => __val,
                                                                     _serde::export::Err(__err)
                                                                     => {
                                                                         return _serde::export::Err(__err);
                                                                     }
                                                                 });
                                    }
                                    __Field::__field1 => {
                                        if _serde::export::Option::is_some(&__field1)
                                           {
                                            return _serde::export::Err(<__A::Error
                                                                           as
                                                                           _serde::de::Error>::duplicate_field("context"));
                                        }
                                        __field1 =
                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<EventContext>(&mut __map)
                                                                     {
                                                                     _serde::export::Ok(__val)
                                                                     => __val,
                                                                     _serde::export::Err(__err)
                                                                     => {
                                                                         return _serde::export::Err(__err);
                                                                     }
                                                                 });
                                    }
                                    __Field::__field2 => {
                                        if _serde::export::Option::is_some(&__field2)
                                           {
                                            return _serde::export::Err(<__A::Error
                                                                           as
                                                                           _serde::de::Error>::duplicate_field("id"));
                                        }
                                        __field2 =
                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<Uuid>(&mut __map)
                                                                     {
                                                                     _serde::export::Ok(__val)
                                                                     => __val,
                                                                     _serde::export::Err(__err)
                                                                     => {
                                                                         return _serde::export::Err(__err);
                                                                     }
                                                                 });
                                    }
                                    _ => {
                                        let _ =
                                            match _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)
                                                {
                                                _serde::export::Ok(__val) =>
                                                __val,
                                                _serde::export::Err(__err) =>
                                                {
                                                    return _serde::export::Err(__err);
                                                }
                                            };
                                    }
                                }
                            }
                            let __field0 =
                                match __field0 {
                                    _serde::export::Some(__field0) =>
                                    __field0,
                                    _serde::export::None =>
                                    match _serde::private::de::missing_field("data")
                                        {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                };
                            let __field1 =
                                match __field1 {
                                    _serde::export::Some(__field1) =>
                                    __field1,
                                    _serde::export::None =>
                                    match _serde::private::de::missing_field("context")
                                        {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                };
                            let __field2 =
                                match __field2 {
                                    _serde::export::Some(__field2) =>
                                    __field2,
                                    _serde::export::None =>
                                    match _serde::private::de::missing_field("id")
                                        {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                };
                            _serde::export::Ok(Event{data: __field0,
                                                     context: __field1,
                                                     id: __field2,})
                        }
                    }
                    const FIELDS: &'static [&'static str] =
                        &["data", "context", "id"];
                    _serde::Deserializer::deserialize_struct(__deserializer,
                                                             "Event", FIELDS,
                                                             __Visitor{marker:
                                                                           _serde::export::PhantomData::<Event<D>>,
                                                                       lifetime:
                                                                           _serde::export::PhantomData,})
                }
            }
        };
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl <D: ::std::fmt::Debug> ::std::fmt::Debug for Event<D> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                Event {
                data: ref __self_0_0,
                context: ref __self_0_1,
                id: ref __self_0_2 } => {
                    let mut debug_trait_builder = f.debug_struct("Event");
                    let _ =
                        debug_trait_builder.field("data", &&(*__self_0_0));
                    let _ =
                        debug_trait_builder.field("context", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("id", &&(*__self_0_2));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl <D> Event<D> where D: EventData {
        /// Create a new event
        pub fn new(data: D, id: Uuid, context: EventContext) -> Self {
            Self{data, context, id,}
        }
        /// Create a new event from some data. `context.time` is set to now, `id` to a new V4 ID
        ///
        /// The rest of the context is left empty
        pub fn from_data(data: D) -> Self {
            Self{data,
                 id: Uuid::new_v4(),
                 context:
                     EventContext{action: None,
                                  subject: None,
                                  time: Utc::now(),},}
        }
        /// Create a copied event with the given ID
        pub fn with_id(self, id: Uuid) -> Self { Self{id, ..self} }
    }
}
mod event_context {
    use chrono::prelude::*;
    use serde_derive::{Deserialize, Serialize};
    use serde_json::Value as JsonValue;
    /// Event context
    ///
    /// Contains metadata for event and, most importantly, the creation time
    pub struct EventContext {
        /// TODO: What is this?
        pub action: Option<String>,
        /// Optional event "subject" or metadata
        pub subject: Option<JsonValue>,
        /// Event creation time
        pub time: DateTime<Utc>,
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _IMPL_SERIALIZE_FOR_EventContext: () =
        {
            #[allow(unknown_lints)]
            #[allow(rust_2018_idioms)]
            extern crate serde as _serde;
            #[allow(unused_macros)]
            macro_rules! try(( $ __expr : expr ) => {
                             match $ __expr {
                             _serde :: export :: Ok ( __val ) => __val ,
                             _serde :: export :: Err ( __err ) => {
                             return _serde :: export :: Err ( __err ) ; } }
                             });
            #[automatically_derived]
            impl _serde::Serialize for EventContext {
                fn serialize<__S>(&self, __serializer: __S)
                 -> _serde::export::Result<__S::Ok, __S::Error> where
                 __S: _serde::Serializer {
                    let mut __serde_state =
                        match _serde::Serializer::serialize_struct(__serializer,
                                                                   "EventContext",
                                                                   false as
                                                                       usize +
                                                                       1 + 1 +
                                                                       1) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        };
                    match _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                                                        "action",
                                                                        &self.action)
                        {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                                                        "subject",
                                                                        &self.subject)
                        {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                                                        "time",
                                                                        &self.time)
                        {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _IMPL_DESERIALIZE_FOR_EventContext: () =
        {
            #[allow(unknown_lints)]
            #[allow(rust_2018_idioms)]
            extern crate serde as _serde;
            #[allow(unused_macros)]
            macro_rules! try(( $ __expr : expr ) => {
                             match $ __expr {
                             _serde :: export :: Ok ( __val ) => __val ,
                             _serde :: export :: Err ( __err ) => {
                             return _serde :: export :: Err ( __err ) ; } }
                             });
            #[automatically_derived]
            impl <'de> _serde::Deserialize<'de> for EventContext {
                fn deserialize<__D>(__deserializer: __D)
                 -> _serde::export::Result<Self, __D::Error> where
                 __D: _serde::Deserializer<'de> {
                    #[allow(non_camel_case_types)]
                    enum __Field { __field0, __field1, __field2, __ignore, }
                    struct __FieldVisitor;
                    impl <'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type
                        Value
                        =
                        __Field;
                        fn expecting(&self,
                                     __formatter:
                                         &mut _serde::export::Formatter)
                         -> _serde::export::fmt::Result {
                            _serde::export::Formatter::write_str(__formatter,
                                                                 "field identifier")
                        }
                        fn visit_u64<__E>(self, __value: u64)
                         -> _serde::export::Result<Self::Value, __E> where
                         __E: _serde::de::Error {
                            match __value {
                                0u64 => _serde::export::Ok(__Field::__field0),
                                1u64 => _serde::export::Ok(__Field::__field1),
                                2u64 => _serde::export::Ok(__Field::__field2),
                                _ =>
                                _serde::export::Err(_serde::de::Error::invalid_value(_serde::de::Unexpected::Unsigned(__value),
                                                                                     &"field index 0 <= i < 3")),
                            }
                        }
                        fn visit_str<__E>(self, __value: &str)
                         -> _serde::export::Result<Self::Value, __E> where
                         __E: _serde::de::Error {
                            match __value {
                                "action" =>
                                _serde::export::Ok(__Field::__field0),
                                "subject" =>
                                _serde::export::Ok(__Field::__field1),
                                "time" =>
                                _serde::export::Ok(__Field::__field2),
                                _ => { _serde::export::Ok(__Field::__ignore) }
                            }
                        }
                        fn visit_bytes<__E>(self, __value: &[u8])
                         -> _serde::export::Result<Self::Value, __E> where
                         __E: _serde::de::Error {
                            match __value {
                                b"action" =>
                                _serde::export::Ok(__Field::__field0),
                                b"subject" =>
                                _serde::export::Ok(__Field::__field1),
                                b"time" =>
                                _serde::export::Ok(__Field::__field2),
                                _ => { _serde::export::Ok(__Field::__ignore) }
                            }
                        }
                    }
                    impl <'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(__deserializer: __D)
                         -> _serde::export::Result<Self, __D::Error> where
                         __D: _serde::Deserializer<'de> {
                            _serde::Deserializer::deserialize_identifier(__deserializer,
                                                                         __FieldVisitor)
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::export::PhantomData<EventContext>,
                        lifetime: _serde::export::PhantomData<&'de ()>,
                    }
                    impl <'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type
                        Value
                        =
                        EventContext;
                        fn expecting(&self,
                                     __formatter:
                                         &mut _serde::export::Formatter)
                         -> _serde::export::fmt::Result {
                            _serde::export::Formatter::write_str(__formatter,
                                                                 "struct EventContext")
                        }
                        #[inline]
                        fn visit_seq<__A>(self, mut __seq: __A)
                         -> _serde::export::Result<Self::Value, __A::Error>
                         where __A: _serde::de::SeqAccess<'de> {
                            let __field0 =
                                match match _serde::de::SeqAccess::next_element::<Option<String>>(&mut __seq)
                                          {
                                          _serde::export::Ok(__val) => __val,
                                          _serde::export::Err(__err) => {
                                              return _serde::export::Err(__err);
                                          }
                                      } {
                                    _serde::export::Some(__value) => __value,
                                    _serde::export::None => {
                                        return _serde::export::Err(_serde::de::Error::invalid_length(0usize,
                                                                                                     &"struct EventContext with 3 elements"));
                                    }
                                };
                            let __field1 =
                                match match _serde::de::SeqAccess::next_element::<Option<JsonValue>>(&mut __seq)
                                          {
                                          _serde::export::Ok(__val) => __val,
                                          _serde::export::Err(__err) => {
                                              return _serde::export::Err(__err);
                                          }
                                      } {
                                    _serde::export::Some(__value) => __value,
                                    _serde::export::None => {
                                        return _serde::export::Err(_serde::de::Error::invalid_length(1usize,
                                                                                                     &"struct EventContext with 3 elements"));
                                    }
                                };
                            let __field2 =
                                match match _serde::de::SeqAccess::next_element::<DateTime<Utc>>(&mut __seq)
                                          {
                                          _serde::export::Ok(__val) => __val,
                                          _serde::export::Err(__err) => {
                                              return _serde::export::Err(__err);
                                          }
                                      } {
                                    _serde::export::Some(__value) => __value,
                                    _serde::export::None => {
                                        return _serde::export::Err(_serde::de::Error::invalid_length(2usize,
                                                                                                     &"struct EventContext with 3 elements"));
                                    }
                                };
                            _serde::export::Ok(EventContext{action: __field0,
                                                            subject: __field1,
                                                            time: __field2,})
                        }
                        #[inline]
                        fn visit_map<__A>(self, mut __map: __A)
                         -> _serde::export::Result<Self::Value, __A::Error>
                         where __A: _serde::de::MapAccess<'de> {
                            let mut __field0:
                                    _serde::export::Option<Option<String>> =
                                _serde::export::None;
                            let mut __field1:
                                    _serde::export::Option<Option<JsonValue>> =
                                _serde::export::None;
                            let mut __field2:
                                    _serde::export::Option<DateTime<Utc>> =
                                _serde::export::None;
                            while let _serde::export::Some(__key) =
                                      match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                          {
                                          _serde::export::Ok(__val) => __val,
                                          _serde::export::Err(__err) => {
                                              return _serde::export::Err(__err);
                                          }
                                      } {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::export::Option::is_some(&__field0)
                                           {
                                            return _serde::export::Err(<__A::Error
                                                                           as
                                                                           _serde::de::Error>::duplicate_field("action"));
                                        }
                                        __field0 =
                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<Option<String>>(&mut __map)
                                                                     {
                                                                     _serde::export::Ok(__val)
                                                                     => __val,
                                                                     _serde::export::Err(__err)
                                                                     => {
                                                                         return _serde::export::Err(__err);
                                                                     }
                                                                 });
                                    }
                                    __Field::__field1 => {
                                        if _serde::export::Option::is_some(&__field1)
                                           {
                                            return _serde::export::Err(<__A::Error
                                                                           as
                                                                           _serde::de::Error>::duplicate_field("subject"));
                                        }
                                        __field1 =
                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<Option<JsonValue>>(&mut __map)
                                                                     {
                                                                     _serde::export::Ok(__val)
                                                                     => __val,
                                                                     _serde::export::Err(__err)
                                                                     => {
                                                                         return _serde::export::Err(__err);
                                                                     }
                                                                 });
                                    }
                                    __Field::__field2 => {
                                        if _serde::export::Option::is_some(&__field2)
                                           {
                                            return _serde::export::Err(<__A::Error
                                                                           as
                                                                           _serde::de::Error>::duplicate_field("time"));
                                        }
                                        __field2 =
                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<DateTime<Utc>>(&mut __map)
                                                                     {
                                                                     _serde::export::Ok(__val)
                                                                     => __val,
                                                                     _serde::export::Err(__err)
                                                                     => {
                                                                         return _serde::export::Err(__err);
                                                                     }
                                                                 });
                                    }
                                    _ => {
                                        let _ =
                                            match _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)
                                                {
                                                _serde::export::Ok(__val) =>
                                                __val,
                                                _serde::export::Err(__err) =>
                                                {
                                                    return _serde::export::Err(__err);
                                                }
                                            };
                                    }
                                }
                            }
                            let __field0 =
                                match __field0 {
                                    _serde::export::Some(__field0) =>
                                    __field0,
                                    _serde::export::None =>
                                    match _serde::private::de::missing_field("action")
                                        {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                };
                            let __field1 =
                                match __field1 {
                                    _serde::export::Some(__field1) =>
                                    __field1,
                                    _serde::export::None =>
                                    match _serde::private::de::missing_field("subject")
                                        {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                };
                            let __field2 =
                                match __field2 {
                                    _serde::export::Some(__field2) =>
                                    __field2,
                                    _serde::export::None =>
                                    match _serde::private::de::missing_field("time")
                                        {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                };
                            _serde::export::Ok(EventContext{action: __field0,
                                                            subject: __field1,
                                                            time: __field2,})
                        }
                    }
                    const FIELDS: &'static [&'static str] =
                        &["action", "subject", "time"];
                    _serde::Deserializer::deserialize_struct(__deserializer,
                                                             "EventContext",
                                                             FIELDS,
                                                             __Visitor{marker:
                                                                           _serde::export::PhantomData::<EventContext>,
                                                                       lifetime:
                                                                           _serde::export::PhantomData,})
                }
            }
        };
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::fmt::Debug for EventContext {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                EventContext {
                action: ref __self_0_0,
                subject: ref __self_0_1,
                time: ref __self_0_2 } => {
                    let mut debug_trait_builder =
                        f.debug_struct("EventContext");
                    let _ =
                        debug_trait_builder.field("action", &&(*__self_0_0));
                    let _ =
                        debug_trait_builder.field("subject", &&(*__self_0_1));
                    let _ =
                        debug_trait_builder.field("time", &&(*__self_0_2));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::clone::Clone for EventContext {
        #[inline]
        fn clone(&self) -> EventContext {
            match *self {
                EventContext {
                action: ref __self_0_0,
                subject: ref __self_0_1,
                time: ref __self_0_2 } =>
                EventContext{action:
                                 ::std::clone::Clone::clone(&(*__self_0_0)),
                             subject:
                                 ::std::clone::Clone::clone(&(*__self_0_1)),
                             time:
                                 ::std::clone::Clone::clone(&(*__self_0_2)),},
            }
        }
    }
}
mod event_handler {
    //! Event handler trait
    use crate::event::Event;
    use crate::store::Store;
    use event_store_derive_internals::EventData;
    /// Event handler trait
    pub trait EventHandler: Sized + EventData {
        /// The method called when an incoming event is received
        fn handle_event(_event: Event<Self>, _saver: &Store) { }
    }
}
mod event_replay {
    use crate::event::Event;
    use crate::event_handler::EventHandler;
    use crate::store::Store;
    use chrono::prelude::*;
    use event_store_derive::*;
    use event_store_derive_internals::EventData;
    use log::{debug, error};
    use serde_derive::*;
    #[event_store(namespace = "_eventstore")]
    pub(crate) struct EventReplayRequested {
        requested_event_namespace: String,
        requested_event_type: String,
        since: DateTime<Utc>,
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_imports)]
    const _IMPL_EVENT_STORE_STRUCT_FOR_EventReplayRequested: () =
        {
            extern crate serde;
            extern crate serde_derive;
            extern crate event_store_derive_internals;
            use serde::ser;
            use serde::de::{Deserialize, Deserializer};
            use serde::ser::{Serialize, Serializer, SerializeMap};
            impl event_store_derive_internals::EventData for
             EventReplayRequested {
                fn event_namespace_and_type() -> &'static str {
                    "_eventstore.EventReplayRequested"
                }
                fn event_namespace() -> &'static str { "_eventstore" }
                fn event_type() -> &'static str { "EventReplayRequested" }
            }
            impl Serialize for EventReplayRequested {
                fn serialize<S>(&self, serializer: S)
                 -> Result<S::Ok, S::Error> where S: Serializer {
                    let mut map = serializer.serialize_map(Some(6usize))?;
                    for (k, v) in self {
                        map.serialize_entry("#field_idents",
                                            self.requested_event_namespace)?;
                        map.serialize_entry("#field_idents",
                                            self.requested_event_type)?;
                        map.serialize_entry("#field_idents", self.since)?;
                    }
                    map.serialize_entry("event_namespace_and_type",
                                        "_eventstore.EventReplayRequested")?;
                    map.serialize_entry("event_namespace", "_eventstore")?;
                    map.serialize_entry("event_type",
                                        "EventReplayRequested")?;
                    map.end()
                }
            }
            impl <'de> serde::Deserialize<'de> for EventReplayRequested {
                fn deserialize<__D>(deserializer: __D)
                 -> serde::export::Result<Self, __D::Error> where
                 __D: serde::Deserializer<'de> {
                    use serde::de;
                    struct EventIdent {
                        event_type: String,
                        event_namespace: String,
                    }
                    #[allow(non_upper_case_globals,
                            unused_attributes,
                            unused_qualifications)]
                    const _IMPL_DESERIALIZE_FOR_EventIdent: () =
                        {
                            #[allow(unknown_lints)]
                            #[allow(rust_2018_idioms)]
                            extern crate serde as _serde;
                            #[allow(unused_macros)]
                            macro_rules! try(( $ __expr : expr ) => {
                                             match $ __expr {
                                             _serde :: export :: Ok ( __val )
                                             => __val , _serde :: export ::
                                             Err ( __err ) => {
                                             return _serde :: export :: Err (
                                             __err ) ; } } });
                            #[automatically_derived]
                            impl <'de> _serde::Deserialize<'de> for EventIdent
                             {
                                fn deserialize<__D>(__deserializer: __D)
                                 -> _serde::export::Result<Self, __D::Error>
                                 where __D: _serde::Deserializer<'de> {
                                    #[allow(non_camel_case_types)]
                                    enum __Field {
                                        __field0,
                                        __field1,
                                        __ignore,
                                    }
                                    struct __FieldVisitor;
                                    impl <'de> _serde::de::Visitor<'de> for
                                     __FieldVisitor {
                                        type
                                        Value
                                        =
                                        __Field;
                                        fn expecting(&self,
                                                     __formatter:
                                                         &mut _serde::export::Formatter)
                                         -> _serde::export::fmt::Result {
                                            _serde::export::Formatter::write_str(__formatter,
                                                                                 "field identifier")
                                        }
                                        fn visit_u64<__E>(self, __value: u64)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            match __value {
                                                0u64 =>
                                                _serde::export::Ok(__Field::__field0),
                                                1u64 =>
                                                _serde::export::Ok(__Field::__field1),
                                                _ =>
                                                _serde::export::Err(_serde::de::Error::invalid_value(_serde::de::Unexpected::Unsigned(__value),
                                                                                                     &"field index 0 <= i < 2")),
                                            }
                                        }
                                        fn visit_str<__E>(self, __value: &str)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            match __value {
                                                "event_type" =>
                                                _serde::export::Ok(__Field::__field0),
                                                "event_namespace" =>
                                                _serde::export::Ok(__Field::__field1),
                                                _ => {
                                                    _serde::export::Ok(__Field::__ignore)
                                                }
                                            }
                                        }
                                        fn visit_bytes<__E>(self,
                                                            __value: &[u8])
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            match __value {
                                                b"event_type" =>
                                                _serde::export::Ok(__Field::__field0),
                                                b"event_namespace" =>
                                                _serde::export::Ok(__Field::__field1),
                                                _ => {
                                                    _serde::export::Ok(__Field::__ignore)
                                                }
                                            }
                                        }
                                    }
                                    impl <'de> _serde::Deserialize<'de> for
                                     __Field {
                                        #[inline]
                                        fn deserialize<__D>(__deserializer:
                                                                __D)
                                         ->
                                             _serde::export::Result<Self,
                                                                    __D::Error>
                                         where
                                         __D: _serde::Deserializer<'de> {
                                            _serde::Deserializer::deserialize_identifier(__deserializer,
                                                                                         __FieldVisitor)
                                        }
                                    }
                                    struct __Visitor<'de> {
                                        marker: _serde::export::PhantomData<EventIdent>,
                                        lifetime: _serde::export::PhantomData<&'de ()>,
                                    }
                                    impl <'de> _serde::de::Visitor<'de> for
                                     __Visitor<'de> {
                                        type
                                        Value
                                        =
                                        EventIdent;
                                        fn expecting(&self,
                                                     __formatter:
                                                         &mut _serde::export::Formatter)
                                         -> _serde::export::fmt::Result {
                                            _serde::export::Formatter::write_str(__formatter,
                                                                                 "struct EventIdent")
                                        }
                                        #[inline]
                                        fn visit_seq<__A>(self,
                                                          mut __seq: __A)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __A::Error>
                                         where
                                         __A: _serde::de::SeqAccess<'de> {
                                            let __field0 =
                                                match match _serde::de::SeqAccess::next_element::<String>(&mut __seq)
                                                          {
                                                          _serde::export::Ok(__val)
                                                          => __val,
                                                          _serde::export::Err(__err)
                                                          => {
                                                              return _serde::export::Err(__err);
                                                          }
                                                      } {
                                                    _serde::export::Some(__value)
                                                    => __value,
                                                    _serde::export::None => {
                                                        return _serde::export::Err(_serde::de::Error::invalid_length(0usize,
                                                                                                                     &"struct EventIdent with 2 elements"));
                                                    }
                                                };
                                            let __field1 =
                                                match match _serde::de::SeqAccess::next_element::<String>(&mut __seq)
                                                          {
                                                          _serde::export::Ok(__val)
                                                          => __val,
                                                          _serde::export::Err(__err)
                                                          => {
                                                              return _serde::export::Err(__err);
                                                          }
                                                      } {
                                                    _serde::export::Some(__value)
                                                    => __value,
                                                    _serde::export::None => {
                                                        return _serde::export::Err(_serde::de::Error::invalid_length(1usize,
                                                                                                                     &"struct EventIdent with 2 elements"));
                                                    }
                                                };
                                            _serde::export::Ok(EventIdent{event_type:
                                                                              __field0,
                                                                          event_namespace:
                                                                              __field1,})
                                        }
                                        #[inline]
                                        fn visit_map<__A>(self,
                                                          mut __map: __A)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __A::Error>
                                         where
                                         __A: _serde::de::MapAccess<'de> {
                                            let mut __field0:
                                                    _serde::export::Option<String> =
                                                _serde::export::None;
                                            let mut __field1:
                                                    _serde::export::Option<String> =
                                                _serde::export::None;
                                            while let _serde::export::Some(__key)
                                                      =
                                                      match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                                          {
                                                          _serde::export::Ok(__val)
                                                          => __val,
                                                          _serde::export::Err(__err)
                                                          => {
                                                              return _serde::export::Err(__err);
                                                          }
                                                      } {
                                                match __key {
                                                    __Field::__field0 => {
                                                        if _serde::export::Option::is_some(&__field0)
                                                           {
                                                            return _serde::export::Err(<__A::Error
                                                                                           as
                                                                                           _serde::de::Error>::duplicate_field("event_type"));
                                                        }
                                                        __field0 =
                                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<String>(&mut __map)
                                                                                     {
                                                                                     _serde::export::Ok(__val)
                                                                                     =>
                                                                                     __val,
                                                                                     _serde::export::Err(__err)
                                                                                     =>
                                                                                     {
                                                                                         return _serde::export::Err(__err);
                                                                                     }
                                                                                 });
                                                    }
                                                    __Field::__field1 => {
                                                        if _serde::export::Option::is_some(&__field1)
                                                           {
                                                            return _serde::export::Err(<__A::Error
                                                                                           as
                                                                                           _serde::de::Error>::duplicate_field("event_namespace"));
                                                        }
                                                        __field1 =
                                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<String>(&mut __map)
                                                                                     {
                                                                                     _serde::export::Ok(__val)
                                                                                     =>
                                                                                     __val,
                                                                                     _serde::export::Err(__err)
                                                                                     =>
                                                                                     {
                                                                                         return _serde::export::Err(__err);
                                                                                     }
                                                                                 });
                                                    }
                                                    _ => {
                                                        let _ =
                                                            match _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)
                                                                {
                                                                _serde::export::Ok(__val)
                                                                => __val,
                                                                _serde::export::Err(__err)
                                                                => {
                                                                    return _serde::export::Err(__err);
                                                                }
                                                            };
                                                    }
                                                }
                                            }
                                            let __field0 =
                                                match __field0 {
                                                    _serde::export::Some(__field0)
                                                    => __field0,
                                                    _serde::export::None =>
                                                    match _serde::private::de::missing_field("event_type")
                                                        {
                                                        _serde::export::Ok(__val)
                                                        => __val,
                                                        _serde::export::Err(__err)
                                                        => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    },
                                                };
                                            let __field1 =
                                                match __field1 {
                                                    _serde::export::Some(__field1)
                                                    => __field1,
                                                    _serde::export::None =>
                                                    match _serde::private::de::missing_field("event_namespace")
                                                        {
                                                        _serde::export::Ok(__val)
                                                        => __val,
                                                        _serde::export::Err(__err)
                                                        => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    },
                                                };
                                            _serde::export::Ok(EventIdent{event_type:
                                                                              __field0,
                                                                          event_namespace:
                                                                              __field1,})
                                        }
                                    }
                                    const FIELDS: &'static [&'static str] =
                                        &["event_type", "event_namespace"];
                                    _serde::Deserializer::deserialize_struct(__deserializer,
                                                                             "EventIdent",
                                                                             FIELDS,
                                                                             __Visitor{marker:
                                                                                           _serde::export::PhantomData::<EventIdent>,
                                                                                       lifetime:
                                                                                           _serde::export::PhantomData,})
                                }
                            }
                        };
                    #[automatically_derived]
                    #[allow(unused_qualifications)]
                    impl ::std::clone::Clone for EventIdent {
                        #[inline]
                        fn clone(&self) -> EventIdent {
                            match *self {
                                EventIdent {
                                event_type: ref __self_0_0,
                                event_namespace: ref __self_0_1 } =>
                                EventIdent{event_type:
                                               ::std::clone::Clone::clone(&(*__self_0_0)),
                                           event_namespace:
                                               ::std::clone::Clone::clone(&(*__self_0_1)),},
                            }
                        }
                    }
                    struct Helper {
                        #[serde(rename = "type")]
                        _event_namespace_and_type: Option<String>,
                        #[serde(flatten)]
                        _event_ident: Option<EventIdent>,
                        requested_event_namespace: String,
                        requested_event_type: String,
                        since: DateTime<Utc>,
                    }
                    #[allow(non_upper_case_globals,
                            unused_attributes,
                            unused_qualifications)]
                    const _IMPL_DESERIALIZE_FOR_Helper: () =
                        {
                            #[allow(unknown_lints)]
                            #[allow(rust_2018_idioms)]
                            extern crate serde as _serde;
                            #[allow(unused_macros)]
                            macro_rules! try(( $ __expr : expr ) => {
                                             match $ __expr {
                                             _serde :: export :: Ok ( __val )
                                             => __val , _serde :: export ::
                                             Err ( __err ) => {
                                             return _serde :: export :: Err (
                                             __err ) ; } } });
                            #[automatically_derived]
                            impl <'de> _serde::Deserialize<'de> for Helper {
                                fn deserialize<__D>(__deserializer: __D)
                                 -> _serde::export::Result<Self, __D::Error>
                                 where __D: _serde::Deserializer<'de> {
                                    #[allow(non_camel_case_types)]
                                    enum __Field<'de> {
                                        __field0,
                                        __field2,
                                        __field3,
                                        __field4,
                                        __other(_serde::private::de::Content<'de>),
                                    }
                                    struct __FieldVisitor;
                                    impl <'de> _serde::de::Visitor<'de> for
                                     __FieldVisitor {
                                        type
                                        Value
                                        =
                                        __Field<'de>;
                                        fn expecting(&self,
                                                     __formatter:
                                                         &mut _serde::export::Formatter)
                                         -> _serde::export::fmt::Result {
                                            _serde::export::Formatter::write_str(__formatter,
                                                                                 "field identifier")
                                        }
                                        fn visit_bool<__E>(self,
                                                           __value: bool)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::Bool(__value)))
                                        }
                                        fn visit_i8<__E>(self, __value: i8)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::I8(__value)))
                                        }
                                        fn visit_i16<__E>(self, __value: i16)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::I16(__value)))
                                        }
                                        fn visit_i32<__E>(self, __value: i32)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::I32(__value)))
                                        }
                                        fn visit_i64<__E>(self, __value: i64)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::I64(__value)))
                                        }
                                        fn visit_u8<__E>(self, __value: u8)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::U8(__value)))
                                        }
                                        fn visit_u16<__E>(self, __value: u16)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::U16(__value)))
                                        }
                                        fn visit_u32<__E>(self, __value: u32)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::U32(__value)))
                                        }
                                        fn visit_u64<__E>(self, __value: u64)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::U64(__value)))
                                        }
                                        fn visit_f32<__E>(self, __value: f32)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::F32(__value)))
                                        }
                                        fn visit_f64<__E>(self, __value: f64)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::F64(__value)))
                                        }
                                        fn visit_char<__E>(self,
                                                           __value: char)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::Char(__value)))
                                        }
                                        fn visit_unit<__E>(self)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            _serde::export::Ok(__Field::__other(_serde::private::de::Content::Unit))
                                        }
                                        fn visit_borrowed_str<__E>(self,
                                                                   __value:
                                                                       &'de str)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            match __value {
                                                "type" =>
                                                _serde::export::Ok(__Field::__field0),
                                                "requested_event_namespace" =>
                                                _serde::export::Ok(__Field::__field2),
                                                "requested_event_type" =>
                                                _serde::export::Ok(__Field::__field3),
                                                "since" =>
                                                _serde::export::Ok(__Field::__field4),
                                                _ => {
                                                    let __value =
                                                        _serde::private::de::Content::Str(__value);
                                                    _serde::export::Ok(__Field::__other(__value))
                                                }
                                            }
                                        }
                                        fn visit_borrowed_bytes<__E>(self,
                                                                     __value:
                                                                         &'de [u8])
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            match __value {
                                                b"type" =>
                                                _serde::export::Ok(__Field::__field0),
                                                b"requested_event_namespace"
                                                =>
                                                _serde::export::Ok(__Field::__field2),
                                                b"requested_event_type" =>
                                                _serde::export::Ok(__Field::__field3),
                                                b"since" =>
                                                _serde::export::Ok(__Field::__field4),
                                                _ => {
                                                    let __value =
                                                        _serde::private::de::Content::Bytes(__value);
                                                    _serde::export::Ok(__Field::__other(__value))
                                                }
                                            }
                                        }
                                        fn visit_str<__E>(self, __value: &str)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            match __value {
                                                "type" =>
                                                _serde::export::Ok(__Field::__field0),
                                                "requested_event_namespace" =>
                                                _serde::export::Ok(__Field::__field2),
                                                "requested_event_type" =>
                                                _serde::export::Ok(__Field::__field3),
                                                "since" =>
                                                _serde::export::Ok(__Field::__field4),
                                                _ => {
                                                    let __value =
                                                        _serde::private::de::Content::String(__value.to_string());
                                                    _serde::export::Ok(__Field::__other(__value))
                                                }
                                            }
                                        }
                                        fn visit_bytes<__E>(self,
                                                            __value: &[u8])
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __E> where
                                         __E: _serde::de::Error {
                                            match __value {
                                                b"type" =>
                                                _serde::export::Ok(__Field::__field0),
                                                b"requested_event_namespace"
                                                =>
                                                _serde::export::Ok(__Field::__field2),
                                                b"requested_event_type" =>
                                                _serde::export::Ok(__Field::__field3),
                                                b"since" =>
                                                _serde::export::Ok(__Field::__field4),
                                                _ => {
                                                    let __value =
                                                        _serde::private::de::Content::ByteBuf(__value.to_vec());
                                                    _serde::export::Ok(__Field::__other(__value))
                                                }
                                            }
                                        }
                                    }
                                    impl <'de> _serde::Deserialize<'de> for
                                     __Field<'de> {
                                        #[inline]
                                        fn deserialize<__D>(__deserializer:
                                                                __D)
                                         ->
                                             _serde::export::Result<Self,
                                                                    __D::Error>
                                         where
                                         __D: _serde::Deserializer<'de> {
                                            _serde::Deserializer::deserialize_identifier(__deserializer,
                                                                                         __FieldVisitor)
                                        }
                                    }
                                    struct __Visitor<'de> {
                                        marker: _serde::export::PhantomData<Helper>,
                                        lifetime: _serde::export::PhantomData<&'de ()>,
                                    }
                                    impl <'de> _serde::de::Visitor<'de> for
                                     __Visitor<'de> {
                                        type
                                        Value
                                        =
                                        Helper;
                                        fn expecting(&self,
                                                     __formatter:
                                                         &mut _serde::export::Formatter)
                                         -> _serde::export::fmt::Result {
                                            _serde::export::Formatter::write_str(__formatter,
                                                                                 "struct Helper")
                                        }
                                        #[inline]
                                        fn visit_map<__A>(self,
                                                          mut __map: __A)
                                         ->
                                             _serde::export::Result<Self::Value,
                                                                    __A::Error>
                                         where
                                         __A: _serde::de::MapAccess<'de> {
                                            let mut __field0:
                                                    _serde::export::Option<Option<String>> =
                                                _serde::export::None;
                                            let mut __field2:
                                                    _serde::export::Option<String> =
                                                _serde::export::None;
                                            let mut __field3:
                                                    _serde::export::Option<String> =
                                                _serde::export::None;
                                            let mut __field4:
                                                    _serde::export::Option<DateTime<Utc>> =
                                                _serde::export::None;
                                            let mut __collect =
                                                _serde::export::Vec::<_serde::export::Option<(_serde::private::de::Content,
                                                                                              _serde::private::de::Content)>>::new();
                                            while let _serde::export::Some(__key)
                                                      =
                                                      match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                                          {
                                                          _serde::export::Ok(__val)
                                                          => __val,
                                                          _serde::export::Err(__err)
                                                          => {
                                                              return _serde::export::Err(__err);
                                                          }
                                                      } {
                                                match __key {
                                                    __Field::__field0 => {
                                                        if _serde::export::Option::is_some(&__field0)
                                                           {
                                                            return _serde::export::Err(<__A::Error
                                                                                           as
                                                                                           _serde::de::Error>::duplicate_field("type"));
                                                        }
                                                        __field0 =
                                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<Option<String>>(&mut __map)
                                                                                     {
                                                                                     _serde::export::Ok(__val)
                                                                                     =>
                                                                                     __val,
                                                                                     _serde::export::Err(__err)
                                                                                     =>
                                                                                     {
                                                                                         return _serde::export::Err(__err);
                                                                                     }
                                                                                 });
                                                    }
                                                    __Field::__field2 => {
                                                        if _serde::export::Option::is_some(&__field2)
                                                           {
                                                            return _serde::export::Err(<__A::Error
                                                                                           as
                                                                                           _serde::de::Error>::duplicate_field("requested_event_namespace"));
                                                        }
                                                        __field2 =
                                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<String>(&mut __map)
                                                                                     {
                                                                                     _serde::export::Ok(__val)
                                                                                     =>
                                                                                     __val,
                                                                                     _serde::export::Err(__err)
                                                                                     =>
                                                                                     {
                                                                                         return _serde::export::Err(__err);
                                                                                     }
                                                                                 });
                                                    }
                                                    __Field::__field3 => {
                                                        if _serde::export::Option::is_some(&__field3)
                                                           {
                                                            return _serde::export::Err(<__A::Error
                                                                                           as
                                                                                           _serde::de::Error>::duplicate_field("requested_event_type"));
                                                        }
                                                        __field3 =
                                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<String>(&mut __map)
                                                                                     {
                                                                                     _serde::export::Ok(__val)
                                                                                     =>
                                                                                     __val,
                                                                                     _serde::export::Err(__err)
                                                                                     =>
                                                                                     {
                                                                                         return _serde::export::Err(__err);
                                                                                     }
                                                                                 });
                                                    }
                                                    __Field::__field4 => {
                                                        if _serde::export::Option::is_some(&__field4)
                                                           {
                                                            return _serde::export::Err(<__A::Error
                                                                                           as
                                                                                           _serde::de::Error>::duplicate_field("since"));
                                                        }
                                                        __field4 =
                                                            _serde::export::Some(match _serde::de::MapAccess::next_value::<DateTime<Utc>>(&mut __map)
                                                                                     {
                                                                                     _serde::export::Ok(__val)
                                                                                     =>
                                                                                     __val,
                                                                                     _serde::export::Err(__err)
                                                                                     =>
                                                                                     {
                                                                                         return _serde::export::Err(__err);
                                                                                     }
                                                                                 });
                                                    }
                                                    __Field::__other(__name)
                                                    => {
                                                        __collect.push(_serde::export::Some((__name,
                                                                                             match _serde::de::MapAccess::next_value(&mut __map)
                                                                                                 {
                                                                                                 _serde::export::Ok(__val)
                                                                                                 =>
                                                                                                 __val,
                                                                                                 _serde::export::Err(__err)
                                                                                                 =>
                                                                                                 {
                                                                                                     return _serde::export::Err(__err);
                                                                                                 }
                                                                                             })));
                                                    }
                                                }
                                            }
                                            let __field0 =
                                                match __field0 {
                                                    _serde::export::Some(__field0)
                                                    => __field0,
                                                    _serde::export::None =>
                                                    match _serde::private::de::missing_field("type")
                                                        {
                                                        _serde::export::Ok(__val)
                                                        => __val,
                                                        _serde::export::Err(__err)
                                                        => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    },
                                                };
                                            let __field2 =
                                                match __field2 {
                                                    _serde::export::Some(__field2)
                                                    => __field2,
                                                    _serde::export::None =>
                                                    match _serde::private::de::missing_field("requested_event_namespace")
                                                        {
                                                        _serde::export::Ok(__val)
                                                        => __val,
                                                        _serde::export::Err(__err)
                                                        => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    },
                                                };
                                            let __field3 =
                                                match __field3 {
                                                    _serde::export::Some(__field3)
                                                    => __field3,
                                                    _serde::export::None =>
                                                    match _serde::private::de::missing_field("requested_event_type")
                                                        {
                                                        _serde::export::Ok(__val)
                                                        => __val,
                                                        _serde::export::Err(__err)
                                                        => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    },
                                                };
                                            let __field4 =
                                                match __field4 {
                                                    _serde::export::Some(__field4)
                                                    => __field4,
                                                    _serde::export::None =>
                                                    match _serde::private::de::missing_field("since")
                                                        {
                                                        _serde::export::Ok(__val)
                                                        => __val,
                                                        _serde::export::Err(__err)
                                                        => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    },
                                                };
                                            let __field1: Option<EventIdent> =
                                                match _serde::de::Deserialize::deserialize(_serde::private::de::FlatMapDeserializer(&mut __collect,
                                                                                                                                    _serde::export::PhantomData))
                                                    {
                                                    _serde::export::Ok(__val)
                                                    => __val,
                                                    _serde::export::Err(__err)
                                                    => {
                                                        return _serde::export::Err(__err);
                                                    }
                                                };
                                            _serde::export::Ok(Helper{_event_namespace_and_type:
                                                                          __field0,
                                                                      _event_ident:
                                                                          __field1,
                                                                      requested_event_namespace:
                                                                          __field2,
                                                                      requested_event_type:
                                                                          __field3,
                                                                      since:
                                                                          __field4,})
                                        }
                                    }
                                    _serde::Deserializer::deserialize_map(__deserializer,
                                                                          __Visitor{marker:
                                                                                        _serde::export::PhantomData::<Helper>,
                                                                                    lifetime:
                                                                                        _serde::export::PhantomData,})
                                }
                            }
                        };
                    #[automatically_derived]
                    #[allow(unused_qualifications)]
                    impl ::std::clone::Clone for Helper {
                        #[inline]
                        fn clone(&self) -> Helper {
                            match *self {
                                Helper {
                                _event_namespace_and_type: ref __self_0_0,
                                _event_ident: ref __self_0_1,
                                requested_event_namespace: ref __self_0_2,
                                requested_event_type: ref __self_0_3,
                                since: ref __self_0_4 } =>
                                Helper{_event_namespace_and_type:
                                           ::std::clone::Clone::clone(&(*__self_0_0)),
                                       _event_ident:
                                           ::std::clone::Clone::clone(&(*__self_0_1)),
                                       requested_event_namespace:
                                           ::std::clone::Clone::clone(&(*__self_0_2)),
                                       requested_event_type:
                                           ::std::clone::Clone::clone(&(*__self_0_3)),
                                       since:
                                           ::std::clone::Clone::clone(&(*__self_0_4)),},
                            }
                        }
                    }
                    let helper =
                        Helper::deserialize(deserializer).map_err(de::Error::custom)?;
                    let ident =
                        if let Some(ident) = helper._event_ident {
                            ident
                        } else if let Some(ns_and_ty) =
                         helper._event_namespace_and_type {
                            let parts =
                                ns_and_ty.split('.').map(|part|
                                                             String::from(part)).collect::<Vec<String>>();
                            EventIdent{event_namespace: parts[0].clone(),
                                       event_type: parts[1].clone(),}
                        } else {
                            return Err(de::Error::custom("No event identifier found"));
                        };
                    if ident.event_type == "EventReplayRequested" &&
                           ident.event_namespace == "_eventstore" {
                        Ok(EventReplayRequested{requested_event_namespace:
                                                    helper.requested_event_namespace,
                                                requested_event_type:
                                                    helper.requested_event_type,
                                                since: helper.since,})
                    } else {
                        Err(de::Error::custom("Incorrect event identifier"))
                    }
                }
            }
        };
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::fmt::Debug for EventReplayRequested {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                EventReplayRequested {
                requested_event_namespace: ref __self_0_0,
                requested_event_type: ref __self_0_1,
                since: ref __self_0_2 } => {
                    let mut debug_trait_builder =
                        f.debug_struct("EventReplayRequested");
                    let _ =
                        debug_trait_builder.field("requested_event_namespace",
                                                  &&(*__self_0_0));
                    let _ =
                        debug_trait_builder.field("requested_event_type",
                                                  &&(*__self_0_1));
                    let _ =
                        debug_trait_builder.field("since", &&(*__self_0_2));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl EventReplayRequested {
        pub(crate) fn from_event<ED>(since: DateTime<Utc>) -> Event<Self>
         where ED: EventData {
            Event::from_data(Self{requested_event_namespace:
                                      ED::event_namespace().to_string(),
                                  requested_event_type:
                                      ED::event_type().to_string(),
                                  since,})
        }
    }
    impl EventHandler for EventReplayRequested {
        fn handle_event(event: Event<Self>, store: &Store) {
            {
                let lvl = ::log::Level::Debug;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level()
                   {
                    ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Event replay received "],
                                                                           &match (&event,)
                                                                                {
                                                                                (arg0,)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Debug::fmt)],
                                                                            }),
                                             lvl,
                                             &("event_store::event_replay",
                                               "event_store::event_replay",
                                               "event-store/src/event_replay.rs",
                                               33u32));
                }
            };
            let store = store.clone();
            tokio::spawn_async(async move
                                   {
                                       let since = event.data.since;
                                       let ns =
                                           event.data.requested_event_namespace;
                                       let ty =
                                           event.data.requested_event_type;
                                       let events =
                                           {
                                               let mut pinned =
                                                   store.read_events_since(&ns,
                                                                           &ty,
                                                                           since);
                                               loop  {
                                                   if let ::std::task::Poll::Ready(x)
                                                          =
                                                          ::std::future::poll_with_tls_waker(unsafe
                                                                                             {
                                                                                                 ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                                             })
                                                          {
                                                       break x ;
                                                   }
                                                   yield
                                               }
                                           };
                                       match events {
                                           Ok(events) => {
                                               {
                                                   let lvl =
                                                       ::log::Level::Debug;
                                                   if lvl <=
                                                          ::log::STATIC_MAX_LEVEL
                                                          &&
                                                          lvl <=
                                                              ::log::max_level()
                                                      {
                                                       ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Found ",
                                                                                                                " events to replay"],
                                                                                                              &match (&events.len(),)
                                                                                                                   {
                                                                                                                   (arg0,)
                                                                                                                   =>
                                                                                                                   [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                ::std::fmt::Display::fmt)],
                                                                                                               }),
                                                                                lvl,
                                                                                &("event_store::event_replay",
                                                                                  "event_store::event_replay",
                                                                                  "event-store/src/event_replay.rs",
                                                                                  47u32));
                                                   }
                                               };
                                               for event in events {
                                                   {
                                                       let lvl =
                                                           ::log::Level::Debug;
                                                       if lvl <=
                                                              ::log::STATIC_MAX_LEVEL
                                                              &&
                                                              lvl <=
                                                                  ::log::max_level()
                                                          {
                                                           ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Replay event "],
                                                                                                                  &match (&event["id"],)
                                                                                                                       {
                                                                                                                       (arg0,)
                                                                                                                       =>
                                                                                                                       [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                    ::std::fmt::Display::fmt)],
                                                                                                                   }),
                                                                                    lvl,
                                                                                    &("event_store::event_replay",
                                                                                      "event_store::event_replay",
                                                                                      "event-store/src/event_replay.rs",
                                                                                      50u32));
                                                       }
                                                   };
                                                   {
                                                       let mut pinned =
                                                           store.emit_value(&ns,
                                                                            &ty,
                                                                            &event);
                                                       loop  {
                                                           if let ::std::task::Poll::Ready(x)
                                                                  =
                                                                  ::std::future::poll_with_tls_waker(unsafe
                                                                                                     {
                                                                                                         ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                                                     })
                                                                  {
                                                               break x ;
                                                           }
                                                           yield
                                                       }
                                                   }.unwrap();
                                               }
                                           }
                                           Err(e) => {
                                               {
                                                   let lvl =
                                                       ::log::Level::Error;
                                                   if lvl <=
                                                          ::log::STATIC_MAX_LEVEL
                                                          &&
                                                          lvl <=
                                                              ::log::max_level()
                                                      {
                                                       ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Failed to retrieve events to replay: "],
                                                                                                              &match (&e.to_string(),)
                                                                                                                   {
                                                                                                                   (arg0,)
                                                                                                                   =>
                                                                                                                   [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                ::std::fmt::Display::fmt)],
                                                                                                               }),
                                                                                lvl,
                                                                                &("event_store::event_replay",
                                                                                  "event_store::event_replay",
                                                                                  "event-store/src/event_replay.rs",
                                                                                  56u32));
                                                   }
                                               };
                                           }
                                       }
                                   });
        }
    }
}
mod store {
    use crate::adapters::{AmqpEmitterAdapter, PgCacheAdapter, PgQuery,
                          PgStoreAdapter, SaveResult, SaveStatus};
    use crate::aggregator::Aggregator;
    use crate::event::Event;
    use crate::store_query::StoreQuery;
    use chrono::prelude::*;
    use event_store_derive_internals::EventData;
    use event_store_derive_internals::Events;
    use log::{debug, trace};
    use serde::Serialize;
    use serde_json::Value as JsonValue;
    use std::fmt::Debug;
    use std::io;
    /// Event store that does not support subscriptions. Passed to [`crate::event_handler::EventHandler`] implementations.
    pub struct Store {
        pub(crate) store: PgStoreAdapter,
        cache: PgCacheAdapter,
        emitter: AmqpEmitterAdapter,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::clone::Clone for Store {
        #[inline]
        fn clone(&self) -> Store {
            match *self {
                Store {
                store: ref __self_0_0,
                cache: ref __self_0_1,
                emitter: ref __self_0_2 } =>
                Store{store: ::std::clone::Clone::clone(&(*__self_0_0)),
                      cache: ::std::clone::Clone::clone(&(*__self_0_1)),
                      emitter: ::std::clone::Clone::clone(&(*__self_0_2)),},
            }
        }
    }
    impl Store {
        /// Create a new non-subscribable store
        pub fn new(store: PgStoreAdapter, cache: PgCacheAdapter,
                   emitter: AmqpEmitterAdapter) -> Self {
            Self{store, cache, emitter,}
        }
        /// Read events from the backing store, producing a reduced result
        pub async fn aggregate<'a, T, QA, E>(&'a self, query_args: &'a QA)
         -> Result<T, io::Error> where E: Events,
         T: Aggregator<E, QA, PgQuery>, QA: Clone + Debug + 'a {
            {
                let lvl = ::log::Level::Debug;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level()
                   {
                    ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Aggregate with arguments "],
                                                                           &match (&query_args,)
                                                                                {
                                                                                (arg0,)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Debug::fmt)],
                                                                            }),
                                             lvl,
                                             &("event_store::store",
                                               "event_store::store",
                                               "event-store/src/store.rs",
                                               41u32));
                }
            };
            let store_query = T::query(query_args.clone());
            let cache_key = store_query.unique_id();
            let debug_cache_key = cache_key.clone();
            let cache_result =
                {
                    let mut pinned = self.cache.read(&cache_key);
                    loop  {
                        if let ::std::task::Poll::Ready(x) =
                               ::std::future::poll_with_tls_waker(unsafe {
                                                                      ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                  }) {
                            break x ;
                        }
                        yield
                    }
                }?;
            {
                let lvl = ::log::Level::Trace;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level()
                   {
                    ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Aggregate cache key ",
                                                                             " result "],
                                                                           &match (&debug_cache_key,
                                                                                   &cache_result)
                                                                                {
                                                                                (arg0,
                                                                                 arg1)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Display::fmt),
                                                                                 ::std::fmt::ArgumentV1::new(arg1,
                                                                                                             ::std::fmt::Debug::fmt)],
                                                                            }),
                                             lvl,
                                             &("event_store::store",
                                               "event_store::store",
                                               "event-store/src/store.rs",
                                               49u32));
                }
            };
            let (initial_state, since) =
                cache_result.map(|res|
                                     (res.0,
                                      Some(res.1))).unwrap_or_else(||
                                                                       (T::default(),
                                                                        None));
            {
                let lvl = ::log::Level::Trace;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level()
                   {
                    ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Aggregate initial state ",
                                                                             ", since "],
                                                                           &match (&initial_state,
                                                                                   &since)
                                                                                {
                                                                                (arg0,
                                                                                 arg1)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Debug::fmt),
                                                                                 ::std::fmt::ArgumentV1::new(arg1,
                                                                                                             ::std::fmt::Debug::fmt)],
                                                                            }),
                                             lvl,
                                             &("event_store::store",
                                               "event_store::store",
                                               "event-store/src/store.rs",
                                               59u32));
                }
            };
            let events =
                {
                    let mut pinned = self.store.read(&store_query, since);
                    loop  {
                        if let ::std::task::Poll::Ready(x) =
                               ::std::future::poll_with_tls_waker(unsafe {
                                                                      ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                  }) {
                            break x ;
                        }
                        yield
                    }
                }?;
            {
                let lvl = ::log::Level::Trace;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level()
                   {
                    ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Read ",
                                                                             " events to aggregate"],
                                                                           &match (&events.len(),)
                                                                                {
                                                                                (arg0,)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Display::fmt)],
                                                                            }),
                                             lvl,
                                             &("event_store::store",
                                               "event_store::store",
                                               "event-store/src/store.rs",
                                               67u32));
                }
            };
            let result = events.iter().fold(initial_state, T::apply_event);
            {
                let mut pinned = self.cache.save(&cache_key, &result);
                loop  {
                    if let ::std::task::Poll::Ready(x) =
                           ::std::future::poll_with_tls_waker(unsafe {
                                                                  ::std::pin::Pin::new_unchecked(&mut pinned)
                                                              }) {
                        break x ;
                    }
                    yield
                }
            }?;
            Ok(result)
        }
        /// Save an event and emit it to other subscribers
        pub async fn save<'a, ED>(&'a self, event: &'a Event<ED>)
         -> SaveResult where ED: EventData + Debug {
            {
                let lvl = ::log::Level::Debug;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level()
                   {
                    ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Save and emit event "],
                                                                           &match (&event,)
                                                                                {
                                                                                (arg0,)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Debug::fmt)],
                                                                            }),
                                             lvl,
                                             &("event_store::store",
                                               "event_store::store",
                                               "event-store/src/store.rs",
                                               81u32));
                }
            };
            self.save_no_emit(&event)?;
            {
                let mut pinned = self.emitter.emit(&event);
                loop  {
                    if let ::std::task::Poll::Ready(x) =
                           ::std::future::poll_with_tls_waker(unsafe {
                                                                  ::std::pin::Pin::new_unchecked(&mut pinned)
                                                              }) {
                        break x ;
                    }
                    yield
                }
            }.map(|_| SaveStatus::Ok)
        }
        /// Save an event without emitting it to other subscribers
        pub fn save_no_emit<'a, ED>(&'a self, event: &'a Event<ED>)
         -> SaveResult where ED: EventData + Debug {
            {
                let lvl = ::log::Level::Debug;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level()
                   {
                    ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Save (no emit) event "],
                                                                           &match (&event,)
                                                                                {
                                                                                (arg0,)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Debug::fmt)],
                                                                            }),
                                             lvl,
                                             &("event_store::store",
                                               "event_store::store",
                                               "event-store/src/store.rs",
                                               93u32));
                }
            };
            self.store.save(&event)
        }
        /// Find the most recent occurrence of an event in the database
        pub async fn last_event<ED>(&self)
         -> Result<Option<Event<ED>>, io::Error> where ED: EventData {
            self.store.last_event::<ED>()
        }
        /// Emit an event to subscribers
        pub async fn emit<'a, ED>(&'a self, event: &'a Event<ED>)
         -> Result<(), io::Error> where ED: EventData {
            {
                let mut pinned = self.emitter.emit(event);
                loop  {
                    if let ::std::task::Poll::Ready(x) =
                           ::std::future::poll_with_tls_waker(unsafe {
                                                                  ::std::pin::Pin::new_unchecked(&mut pinned)
                                                              }) {
                        break x ;
                    }
                    yield
                }
            }
        }
        pub(crate) async fn emit_value<'a,
                                       V>(&'a self, event_type: &'a str,
                                          event_namespace: &'a str,
                                          data: &'a V)
         -> Result<(), io::Error> where V: Serialize {
            {
                let mut pinned =
                    self.emitter.emit_value(event_type, event_namespace,
                                            data);
                loop  {
                    if let ::std::task::Poll::Ready(x) =
                           ::std::future::poll_with_tls_waker(unsafe {
                                                                  ::std::pin::Pin::new_unchecked(&mut pinned)
                                                              }) {
                        break x ;
                    }
                    yield
                }
            }
        }
        /// Read all events since a given time
        pub async fn read_events_since<'a>(&'a self, event_namespace: &'a str,
                                           event_type: &'a str,
                                           since: DateTime<Utc>)
         -> Result<Vec<JsonValue>, io::Error> {
            {
                let mut pinned =
                    self.store.read_events_since(event_namespace, event_type,
                                                 since);
                loop  {
                    if let ::std::task::Poll::Ready(x) =
                           ::std::future::poll_with_tls_waker(unsafe {
                                                                  ::std::pin::Pin::new_unchecked(&mut pinned)
                                                              }) {
                        break x ;
                    }
                    yield
                }
            }
        }
    }
}
mod store_query {
    /// A query to be passed to the store
    ///
    /// This trait must be implemented for whichever type you want to pass to a particular store. See
    /// impls below for examples.
    pub trait StoreQuery {
        /// You must return a unique identifier based on the query you are performing. This identifier
        /// will then be used to identify the cache and optimize the aggregations using memoization
        fn unique_id(&self)
        -> String;
    }
}
mod subscribable_store {
    use crate::adapters::{AmqpEmitterAdapter, PgCacheAdapter, PgQuery,
                          PgStoreAdapter, SaveResult};
    use crate::aggregator::Aggregator;
    use crate::event::Event;
    use crate::event_handler::EventHandler;
    use crate::event_replay::EventReplayRequested;
    use crate::store::Store;
    use crate::subscribe_options::SubscribeOptions;
    use chrono::prelude::*;
    use event_store_derive_internals::EventData;
    use event_store_derive_internals::Events;
    use log::info;
    use std::fmt::Debug;
    use std::io;
    /// The main event store struct
    pub struct SubscribableStore {
        emitter: AmqpEmitterAdapter,
        inner_store: Store,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::clone::Clone for SubscribableStore {
        #[inline]
        fn clone(&self) -> SubscribableStore {
            match *self {
                SubscribableStore {
                emitter: ref __self_0_0, inner_store: ref __self_0_1 } =>
                SubscribableStore{emitter:
                                      ::std::clone::Clone::clone(&(*__self_0_0)),
                                  inner_store:
                                      ::std::clone::Clone::clone(&(*__self_0_1)),},
            }
        }
    }
    impl SubscribableStore {
        /// Create a new event store with the given store, cache and emitter adapters
        pub async fn new(store: PgStoreAdapter, cache: PgCacheAdapter,
                         emitter: AmqpEmitterAdapter)
         -> Result<Self, io::Error> {
            let inner_store = Store::new(store, cache, emitter.clone());
            let store = Self{inner_store, emitter,};
            {
                let mut pinned =
                    store.subscribe::<EventReplayRequested>(SubscribeOptions{replay_previous_events:
                                                                                 false,
                                                                             save_on_receive:
                                                                                 false,});
                loop  {
                    if let ::std::task::Poll::Ready(x) =
                           ::std::future::poll_with_tls_waker(unsafe {
                                                                  ::std::pin::Pin::new_unchecked(&mut pinned)
                                                              }) {
                        break x ;
                    }
                    yield
                }
            }?;
            Ok(store)
        }
        /// Fetch an entity from the store by aggregating over matching events
        pub async fn aggregate<'a, T, QA, E>(&'a self, query_args: &'a QA)
         -> Result<T, io::Error> where E: Events,
         T: Aggregator<E, QA, PgQuery>, QA: Clone + Debug + 'a {
            let res: T =
                {
                    let mut pinned =
                        self.inner_store.aggregate::<'a, T, QA,
                                                     E>(&query_args);
                    loop  {
                        if let ::std::task::Poll::Ready(x) =
                               ::std::future::poll_with_tls_waker(unsafe {
                                                                      ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                  }) {
                            break x ;
                        }
                        yield
                    }
                }?;
            Ok(res)
        }
        /// Save an event to the store, emitting it to other listeners
        pub async fn save<'a, ED>(&'a self, event: &'a Event<ED>)
         -> SaveResult where ED: EventData + Debug {
            {
                let mut pinned = self.inner_store.save(event);
                loop  {
                    if let ::std::task::Poll::Ready(x) =
                           ::std::future::poll_with_tls_waker(unsafe {
                                                                  ::std::pin::Pin::new_unchecked(&mut pinned)
                                                              }) {
                        break x ;
                    }
                    yield
                }
            }
        }
        /// Save an event without emitting it
        pub fn save_no_emit<'a, ED>(&'a self, event: &'a Event<ED>)
         -> SaveResult where ED: EventData + Debug {
            self.inner_store.save_no_emit(event)
        }
        /// Subscribe to incoming events matching the namespace and type in `ED`
        pub async fn subscribe<'a, ED>(&'a self, options: SubscribeOptions)
         -> Result<(), io::Error> where ED: EventHandler + Debug + Send {
            {
                let lvl = ::log::Level::Info;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level()
                   {
                    ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Starting subscription to "],
                                                                           &match (&ED::event_namespace_and_type(),)
                                                                                {
                                                                                (arg0,)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Display::fmt)],
                                                                            }),
                                             lvl,
                                             &("event_store::subscribable_store",
                                               "event_store::subscribable_store",
                                               "event-store/src/subscribable_store.rs",
                                               78u32));
                }
            };
            let inner_store = self.inner_store.clone();
            {
                let mut pinned =
                    self.emitter.subscribe::<ED>(inner_store, options);
                loop  {
                    if let ::std::task::Poll::Ready(x) =
                           ::std::future::poll_with_tls_waker(unsafe {
                                                                  ::std::pin::Pin::new_unchecked(&mut pinned)
                                                              }) {
                        break x ;
                    }
                    yield
                }
            }?;
            let last =
                {
                    let mut pinned = self.inner_store.last_event::<ED>();
                    loop  {
                        if let ::std::task::Poll::Ready(x) =
                               ::std::future::poll_with_tls_waker(unsafe {
                                                                      ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                  }) {
                            break x ;
                        }
                        yield
                    }
                }?;
            let replay =
                EventReplayRequested::from_event::<ED>(last.map(|e|
                                                                    e.context.time).unwrap_or_else(||
                                                                                                       Utc.ymd(1970,
                                                                                                               1,
                                                                                                               1).and_hms(0,
                                                                                                                          0,
                                                                                                                          0)));
            {
                let lvl = ::log::Level::Info;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level()
                   {
                    ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Emit replay request for event "],
                                                                           &match (&ED::event_namespace_and_type(),)
                                                                                {
                                                                                (arg0,)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Display::fmt)],
                                                                            }),
                                             lvl,
                                             &("event_store::subscribable_store",
                                               "event_store::subscribable_store",
                                               "event-store/src/subscribable_store.rs",
                                               94u32));
                }
            };
            {
                let mut pinned = self.inner_store.emit(&replay);
                loop  {
                    if let ::std::task::Poll::Ready(x) =
                           ::std::future::poll_with_tls_waker(unsafe {
                                                                  ::std::pin::Pin::new_unchecked(&mut pinned)
                                                              }) {
                        break x ;
                    }
                    yield
                }
            }
        }
        /// Return a reference to the internal backing store. This is a dangerous method and should not
        /// be used in production code.
        pub fn internals_get_store(&self) -> &Store { &self.inner_store }
    }
}
mod subscribe_options {
    /// Subscribe options
    pub struct SubscribeOptions {
        /// Whether to emit an event replay request when a subscription is started
        pub replay_previous_events: bool,
        /// Whether to save the event when it is received, or just pass it to the handler
        pub save_on_receive: bool,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::fmt::Debug for SubscribeOptions {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                SubscribeOptions {
                replay_previous_events: ref __self_0_0,
                save_on_receive: ref __self_0_1 } => {
                    let mut debug_trait_builder =
                        f.debug_struct("SubscribeOptions");
                    let _ =
                        debug_trait_builder.field("replay_previous_events",
                                                  &&(*__self_0_0));
                    let _ =
                        debug_trait_builder.field("save_on_receive",
                                                  &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::clone::Clone for SubscribeOptions {
        #[inline]
        fn clone(&self) -> SubscribeOptions {
            match *self {
                SubscribeOptions {
                replay_previous_events: ref __self_0_0,
                save_on_receive: ref __self_0_1 } =>
                SubscribeOptions{replay_previous_events:
                                     ::std::clone::Clone::clone(&(*__self_0_0)),
                                 save_on_receive:
                                     ::std::clone::Clone::clone(&(*__self_0_1)),},
            }
        }
    }
    impl Default for SubscribeOptions {
        fn default() -> Self {
            Self{replay_previous_events: true, save_on_receive: true,}
        }
    }
}
pub mod adapters {
    //! Backing store adapters for event storage, caching and subscriptions
    mod cache {
        use chrono::prelude::*;
        mod pg {
            use super::CacheResult;
            use chrono::prelude::*;
            use log::{debug, trace};
            use r2d2::Pool;
            use r2d2_postgres::PostgresConnectionManager;
            use serde::de::DeserializeOwned;
            use serde::Serialize;
            use serde_json::from_value;
            use serde_json::to_value;
            use std::fmt::Debug;
            use std::io;
            const INIT_QUERIES: &'static str =
                r#"
-- Create UUID extension just in case
create extension if not exists "uuid-ossp";

-- Create cache table if it doesn't already exist
create table if not exists aggregate_cache(
    id varchar(64) not null,
    data jsonb not null,
    time timestamp with time zone,
    primary key(id)
);

create index if not exists cache_time on aggregate_cache (time desc);
"#;
            /// Postgres-backed cache adapter
            pub struct PgCacheAdapter {
                conn: Pool<PostgresConnectionManager>,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::clone::Clone for PgCacheAdapter {
                #[inline]
                fn clone(&self) -> PgCacheAdapter {
                    match *self {
                        PgCacheAdapter { conn: ref __self_0_0 } =>
                        PgCacheAdapter{conn:
                                           ::std::clone::Clone::clone(&(*__self_0_0)),},
                    }
                }
            }
            impl PgCacheAdapter {
                /// Create a new PG-backed cache adapter instance
                ///
                /// This will attempt to create the cache table if it does not already exist
                pub async fn new(conn: Pool<PostgresConnectionManager>)
                 -> Result<Self, io::Error> {
                    conn.get().map_err(|e|
                                           io::Error::new(io::ErrorKind::Other,
                                                          e.to_string()))?.batch_execute(INIT_QUERIES)?;
                    Ok(Self{conn,})
                }
                /// Read an item from the cache by key, parsing to type `T`
                pub async fn read<'a, T>(&'a self, key: &'a str)
                 -> Result<Option<CacheResult<T>>, io::Error> where
                 T: DeserializeOwned + Debug {
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                               lvl <= ::log::max_level() {
                            ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Cache read key "],
                                                                                   &match (&key,)
                                                                                        {
                                                                                        (arg0,)
                                                                                        =>
                                                                                        [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                     ::std::fmt::Display::fmt)],
                                                                                    }),
                                                     lvl,
                                                     &("event_store::adapters::cache::pg",
                                                       "event_store::adapters::cache::pg",
                                                       "event-store/src/adapters/cache/pg.rs",
                                                       51u32));
                        }
                    };
                    self.conn.get().unwrap().query("select data, time from aggregate_cache where id = $1 limit 1",
                                                   &[&key]).map(|rows|
                                                                    {
                                                                        let res =
                                                                            if rows.len()
                                                                                   !=
                                                                                   1
                                                                               {
                                                                                None
                                                                            } else {
                                                                                let row =
                                                                                    rows.get(0);
                                                                                let utc:
                                                                                        DateTime<Utc> =
                                                                                    row.get(1);
                                                                                Some((from_value(row.get(0)).map(|decoded:
                                                                                                                      T|
                                                                                                                     decoded).expect("Cant decode the cached entity"),
                                                                                      utc))
                                                                            };
                                                                        {
                                                                            let lvl =
                                                                                ::log::Level::Trace;
                                                                            if lvl
                                                                                   <=
                                                                                   ::log::STATIC_MAX_LEVEL
                                                                                   &&
                                                                                   lvl
                                                                                       <=
                                                                                       ::log::max_level()
                                                                               {
                                                                                ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Cache read result "],
                                                                                                                                       &match (&res,)
                                                                                                                                            {
                                                                                                                                            (arg0,)
                                                                                                                                            =>
                                                                                                                                            [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                         ::std::fmt::Debug::fmt)],
                                                                                                                                        }),
                                                                                                         lvl,
                                                                                                         &("event_store::adapters::cache::pg",
                                                                                                           "event_store::adapters::cache::pg",
                                                                                                           "event-store/src/adapters/cache/pg.rs",
                                                                                                           76u32));
                                                                            }
                                                                        };
                                                                        res
                                                                    }).map_err(|e|
                                                                                   e.into())
                }
                /// Save an event into the cache
                pub async fn save<'a, V>(&'a self, key: &'a str, value: &'a V)
                 -> Result<(), io::Error> where V: Serialize + Debug {
                    {
                        let lvl = ::log::Level::Debug;
                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                               lvl <= ::log::max_level() {
                            ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Cache aggregate result under key ",
                                                                                     ": "],
                                                                                   &match (&key,
                                                                                           &value)
                                                                                        {
                                                                                        (arg0,
                                                                                         arg1)
                                                                                        =>
                                                                                        [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                     ::std::fmt::Display::fmt),
                                                                                         ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                     ::std::fmt::Debug::fmt)],
                                                                                    }),
                                                     lvl,
                                                     &("event_store::adapters::cache::pg",
                                                       "event_store::adapters::cache::pg",
                                                       "event-store/src/adapters/cache/pg.rs",
                                                       88u32));
                        }
                    };
                    self.conn.get().unwrap().execute(r#"insert into aggregate_cache (id, data, time)
                    values ($1, $2, now())
                    on conflict (id)
                    do update set data = excluded.data, time = now() returning data"#,
                                                     &[&key,
                                                       &to_value(value).expect("To value")]).map(|_|
                                                                                                     ()).map_err(|e|
                                                                                                                     e.into())
                }
            }
        }
        mod redis {
            #![allow(unused)]
            use super::CacheResult;
            use chrono::{DateTime, Utc};
            use log::error;
            use redis::{Client, Commands, Connection};
            use serde::de::DeserializeOwned;
            use serde::Serialize;
            use serde_json::{from_str, to_string};
            use std::io;
            pub struct RedisCacheItem<D> {
                data: D,
                time: DateTime<Utc>,
            }
            #[allow(non_upper_case_globals,
                    unused_attributes,
                    unused_qualifications)]
            const _IMPL_SERIALIZE_FOR_RedisCacheItem: () =
                {
                    #[allow(unknown_lints)]
                    #[allow(rust_2018_idioms)]
                    extern crate serde as _serde;
                    #[allow(unused_macros)]
                    macro_rules! try(( $ __expr : expr ) => {
                                     match $ __expr {
                                     _serde :: export :: Ok ( __val ) => __val
                                     , _serde :: export :: Err ( __err ) => {
                                     return _serde :: export :: Err ( __err )
                                     ; } } });
                    #[automatically_derived]
                    impl <D> _serde::Serialize for RedisCacheItem<D> where
                     D: _serde::Serialize {
                        fn serialize<__S>(&self, __serializer: __S)
                         -> _serde::export::Result<__S::Ok, __S::Error> where
                         __S: _serde::Serializer {
                            let mut __serde_state =
                                match _serde::Serializer::serialize_struct(__serializer,
                                                                           "RedisCacheItem",
                                                                           false
                                                                               as
                                                                               usize
                                                                               +
                                                                               1
                                                                               +
                                                                               1)
                                    {
                                    _serde::export::Ok(__val) => __val,
                                    _serde::export::Err(__err) => {
                                        return _serde::export::Err(__err);
                                    }
                                };
                            match _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                                                                "data",
                                                                                &self.data)
                                {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            };
                            match _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                                                                "time",
                                                                                &self.time)
                                {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            };
                            _serde::ser::SerializeStruct::end(__serde_state)
                        }
                    }
                };
            #[allow(non_upper_case_globals,
                    unused_attributes,
                    unused_qualifications)]
            const _IMPL_DESERIALIZE_FOR_RedisCacheItem: () =
                {
                    #[allow(unknown_lints)]
                    #[allow(rust_2018_idioms)]
                    extern crate serde as _serde;
                    #[allow(unused_macros)]
                    macro_rules! try(( $ __expr : expr ) => {
                                     match $ __expr {
                                     _serde :: export :: Ok ( __val ) => __val
                                     , _serde :: export :: Err ( __err ) => {
                                     return _serde :: export :: Err ( __err )
                                     ; } } });
                    #[automatically_derived]
                    impl <'de, D> _serde::Deserialize<'de> for
                     RedisCacheItem<D> where D: _serde::Deserialize<'de> {
                        fn deserialize<__D>(__deserializer: __D)
                         -> _serde::export::Result<Self, __D::Error> where
                         __D: _serde::Deserializer<'de> {
                            #[allow(non_camel_case_types)]
                            enum __Field { __field0, __field1, __ignore, }
                            struct __FieldVisitor;
                            impl <'de> _serde::de::Visitor<'de> for
                             __FieldVisitor {
                                type
                                Value
                                =
                                __Field;
                                fn expecting(&self,
                                             __formatter:
                                                 &mut _serde::export::Formatter)
                                 -> _serde::export::fmt::Result {
                                    _serde::export::Formatter::write_str(__formatter,
                                                                         "field identifier")
                                }
                                fn visit_u64<__E>(self, __value: u64)
                                 -> _serde::export::Result<Self::Value, __E>
                                 where __E: _serde::de::Error {
                                    match __value {
                                        0u64 =>
                                        _serde::export::Ok(__Field::__field0),
                                        1u64 =>
                                        _serde::export::Ok(__Field::__field1),
                                        _ =>
                                        _serde::export::Err(_serde::de::Error::invalid_value(_serde::de::Unexpected::Unsigned(__value),
                                                                                             &"field index 0 <= i < 2")),
                                    }
                                }
                                fn visit_str<__E>(self, __value: &str)
                                 -> _serde::export::Result<Self::Value, __E>
                                 where __E: _serde::de::Error {
                                    match __value {
                                        "data" =>
                                        _serde::export::Ok(__Field::__field0),
                                        "time" =>
                                        _serde::export::Ok(__Field::__field1),
                                        _ => {
                                            _serde::export::Ok(__Field::__ignore)
                                        }
                                    }
                                }
                                fn visit_bytes<__E>(self, __value: &[u8])
                                 -> _serde::export::Result<Self::Value, __E>
                                 where __E: _serde::de::Error {
                                    match __value {
                                        b"data" =>
                                        _serde::export::Ok(__Field::__field0),
                                        b"time" =>
                                        _serde::export::Ok(__Field::__field1),
                                        _ => {
                                            _serde::export::Ok(__Field::__ignore)
                                        }
                                    }
                                }
                            }
                            impl <'de> _serde::Deserialize<'de> for __Field {
                                #[inline]
                                fn deserialize<__D>(__deserializer: __D)
                                 -> _serde::export::Result<Self, __D::Error>
                                 where __D: _serde::Deserializer<'de> {
                                    _serde::Deserializer::deserialize_identifier(__deserializer,
                                                                                 __FieldVisitor)
                                }
                            }
                            struct __Visitor<'de, D> where
                                   D: _serde::Deserialize<'de> {
                                marker: _serde::export::PhantomData<RedisCacheItem<D>>,
                                lifetime: _serde::export::PhantomData<&'de ()>,
                            }
                            impl <'de, D> _serde::de::Visitor<'de> for
                             __Visitor<'de, D> where
                             D: _serde::Deserialize<'de> {
                                type
                                Value
                                =
                                RedisCacheItem<D>;
                                fn expecting(&self,
                                             __formatter:
                                                 &mut _serde::export::Formatter)
                                 -> _serde::export::fmt::Result {
                                    _serde::export::Formatter::write_str(__formatter,
                                                                         "struct RedisCacheItem")
                                }
                                #[inline]
                                fn visit_seq<__A>(self, mut __seq: __A)
                                 ->
                                     _serde::export::Result<Self::Value,
                                                            __A::Error> where
                                 __A: _serde::de::SeqAccess<'de> {
                                    let __field0 =
                                        match match _serde::de::SeqAccess::next_element::<D>(&mut __seq)
                                                  {
                                                  _serde::export::Ok(__val) =>
                                                  __val,
                                                  _serde::export::Err(__err)
                                                  => {
                                                      return _serde::export::Err(__err);
                                                  }
                                              } {
                                            _serde::export::Some(__value) =>
                                            __value,
                                            _serde::export::None => {
                                                return _serde::export::Err(_serde::de::Error::invalid_length(0usize,
                                                                                                             &"struct RedisCacheItem with 2 elements"));
                                            }
                                        };
                                    let __field1 =
                                        match match _serde::de::SeqAccess::next_element::<DateTime<Utc>>(&mut __seq)
                                                  {
                                                  _serde::export::Ok(__val) =>
                                                  __val,
                                                  _serde::export::Err(__err)
                                                  => {
                                                      return _serde::export::Err(__err);
                                                  }
                                              } {
                                            _serde::export::Some(__value) =>
                                            __value,
                                            _serde::export::None => {
                                                return _serde::export::Err(_serde::de::Error::invalid_length(1usize,
                                                                                                             &"struct RedisCacheItem with 2 elements"));
                                            }
                                        };
                                    _serde::export::Ok(RedisCacheItem{data:
                                                                          __field0,
                                                                      time:
                                                                          __field1,})
                                }
                                #[inline]
                                fn visit_map<__A>(self, mut __map: __A)
                                 ->
                                     _serde::export::Result<Self::Value,
                                                            __A::Error> where
                                 __A: _serde::de::MapAccess<'de> {
                                    let mut __field0:
                                            _serde::export::Option<D> =
                                        _serde::export::None;
                                    let mut __field1:
                                            _serde::export::Option<DateTime<Utc>> =
                                        _serde::export::None;
                                    while let _serde::export::Some(__key) =
                                              match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                                  {
                                                  _serde::export::Ok(__val) =>
                                                  __val,
                                                  _serde::export::Err(__err)
                                                  => {
                                                      return _serde::export::Err(__err);
                                                  }
                                              } {
                                        match __key {
                                            __Field::__field0 => {
                                                if _serde::export::Option::is_some(&__field0)
                                                   {
                                                    return _serde::export::Err(<__A::Error
                                                                                   as
                                                                                   _serde::de::Error>::duplicate_field("data"));
                                                }
                                                __field0 =
                                                    _serde::export::Some(match _serde::de::MapAccess::next_value::<D>(&mut __map)
                                                                             {
                                                                             _serde::export::Ok(__val)
                                                                             =>
                                                                             __val,
                                                                             _serde::export::Err(__err)
                                                                             =>
                                                                             {
                                                                                 return _serde::export::Err(__err);
                                                                             }
                                                                         });
                                            }
                                            __Field::__field1 => {
                                                if _serde::export::Option::is_some(&__field1)
                                                   {
                                                    return _serde::export::Err(<__A::Error
                                                                                   as
                                                                                   _serde::de::Error>::duplicate_field("time"));
                                                }
                                                __field1 =
                                                    _serde::export::Some(match _serde::de::MapAccess::next_value::<DateTime<Utc>>(&mut __map)
                                                                             {
                                                                             _serde::export::Ok(__val)
                                                                             =>
                                                                             __val,
                                                                             _serde::export::Err(__err)
                                                                             =>
                                                                             {
                                                                                 return _serde::export::Err(__err);
                                                                             }
                                                                         });
                                            }
                                            _ => {
                                                let _ =
                                                    match _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)
                                                        {
                                                        _serde::export::Ok(__val)
                                                        => __val,
                                                        _serde::export::Err(__err)
                                                        => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    };
                                            }
                                        }
                                    }
                                    let __field0 =
                                        match __field0 {
                                            _serde::export::Some(__field0) =>
                                            __field0,
                                            _serde::export::None =>
                                            match _serde::private::de::missing_field("data")
                                                {
                                                _serde::export::Ok(__val) =>
                                                __val,
                                                _serde::export::Err(__err) =>
                                                {
                                                    return _serde::export::Err(__err);
                                                }
                                            },
                                        };
                                    let __field1 =
                                        match __field1 {
                                            _serde::export::Some(__field1) =>
                                            __field1,
                                            _serde::export::None =>
                                            match _serde::private::de::missing_field("time")
                                                {
                                                _serde::export::Ok(__val) =>
                                                __val,
                                                _serde::export::Err(__err) =>
                                                {
                                                    return _serde::export::Err(__err);
                                                }
                                            },
                                        };
                                    _serde::export::Ok(RedisCacheItem{data:
                                                                          __field0,
                                                                      time:
                                                                          __field1,})
                                }
                            }
                            const FIELDS: &'static [&'static str] =
                                &["data", "time"];
                            _serde::Deserializer::deserialize_struct(__deserializer,
                                                                     "RedisCacheItem",
                                                                     FIELDS,
                                                                     __Visitor{marker:
                                                                                   _serde::export::PhantomData::<RedisCacheItem<D>>,
                                                                               lifetime:
                                                                                   _serde::export::PhantomData,})
                        }
                    }
                };
            /// Redis cache adapter
            pub struct RedisCacheAdapter {
                client: Client,
                conn: Connection,
            }
            impl Clone for RedisCacheAdapter {
                fn clone(&self) -> Self {
                    let client = self.client.clone();
                    let conn =
                        client.get_connection().expect("Could not clone Redis cache adapter");
                    Self{client, conn,}
                }
            }
            impl RedisCacheAdapter {
                /// Create a new Redis-backed cache from a Redis client handle
                ///
                /// A connection to Redis is created from the client each time the adapter is created **or
                /// cloned**. It should be cloned as little as possible.
                pub async fn new(client: Client) -> Result<Self, io::Error> {
                    let conn =
                        client.get_connection().expect("Could not get Redis connection");
                    Ok(Self{client, conn,})
                }
                pub async fn set<'a, V>(&'a self, id: String, data: V)
                 -> Result<(), String> where V: Serialize + 'a {
                    let time = Utc::now();
                    let value: String =
                        to_string(&RedisCacheItem{time,
                                                  data,}).expect("Failed to serialize Redis cache item");
                    self.conn.set(id,
                                  value).map_err(|err|
                                                     {
                                                         {
                                                             let lvl =
                                                                 ::log::Level::Error;
                                                             if lvl <=
                                                                    ::log::STATIC_MAX_LEVEL
                                                                    &&
                                                                    lvl <=
                                                                        ::log::max_level()
                                                                {
                                                                 ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Failed to set cache item: "],
                                                                                                                        &match (&err,)
                                                                                                                             {
                                                                                                                             (arg0,)
                                                                                                                             =>
                                                                                                                             [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                          ::std::fmt::Debug::fmt)],
                                                                                                                         }),
                                                                                          lvl,
                                                                                          &("event_store::adapters::cache::redis",
                                                                                            "event_store::adapters::cache::redis",
                                                                                            "event-store/src/adapters/cache/redis.rs",
                                                                                            60u32));
                                                             }
                                                         };
                                                         "Failed to set cache item".into()
                                                     })
                }
                pub async fn get<T>(&self, key: String)
                 -> Result<Option<CacheResult<T>>, String> where
                 T: DeserializeOwned {
                    let value: Option<String> = self.conn.get(key).unwrap();
                    Ok(value.map(|value|
                                     {
                                         let parsed: RedisCacheItem<T> =
                                             from_str(&value).unwrap();
                                         (parsed.data, parsed.time)
                                     }))
                }
            }
        }
        /// Result of a cache search
        pub type CacheResult<T> = (T, DateTime<Utc>);
        pub use self::pg::PgCacheAdapter;
        pub use self::redis::RedisCacheAdapter;
    }
    mod emitter {
        mod amqp {
            use crate::adapters::SaveStatus;
            use crate::event::Event;
            use crate::event_handler::EventHandler;
            use crate::internals::forward;
            use crate::store::Store;
            use crate::subscribe_options::SubscribeOptions;
            use event_store_derive_internals::EventData;
            use futures::Future;
            use lapin_futures::channel::{BasicConsumeOptions, BasicProperties,
                                         BasicPublishOptions, Channel,
                                         ExchangeDeclareOptions,
                                         QueueBindOptions,
                                         QueueDeclareOptions};
            use lapin_futures::client::{Client, ConnectionOptions};
            use lapin_futures::consumer::Consumer;
            use lapin_futures::queue::Queue;
            use lapin_futures::types::FieldTable;
            use log::{debug, error, info, trace};
            use serde::Serialize;
            use serde_json::Value as JsonValue;
            use std::fmt::Debug;
            use std::io;
            use std::net::SocketAddr;
            use tokio::net::TcpStream;
            use tokio_async_await::stream::StreamExt;
            /// AMQP-backed emitter/subscriber
            pub struct AmqpEmitterAdapter {
                channel: Channel<TcpStream>,
                exchange: String,
                store_namespace: String,
                uri: SocketAddr,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::clone::Clone for AmqpEmitterAdapter {
                #[inline]
                fn clone(&self) -> AmqpEmitterAdapter {
                    match *self {
                        AmqpEmitterAdapter {
                        channel: ref __self_0_0,
                        exchange: ref __self_0_1,
                        store_namespace: ref __self_0_2,
                        uri: ref __self_0_3 } =>
                        AmqpEmitterAdapter{channel:
                                               ::std::clone::Clone::clone(&(*__self_0_0)),
                                           exchange:
                                               ::std::clone::Clone::clone(&(*__self_0_1)),
                                           store_namespace:
                                               ::std::clone::Clone::clone(&(*__self_0_2)),
                                           uri:
                                               ::std::clone::Clone::clone(&(*__self_0_3)),},
                    }
                }
            }
            impl AmqpEmitterAdapter {
                /// Create a new AMQP emitter/subscriber
                pub async fn new(uri: SocketAddr, exchange: String,
                                 store_namespace: String)
                 -> Result<Self, io::Error> {
                    let channel =
                        {
                            let mut pinned = amqp_connect(uri, &exchange);
                            loop  {
                                if let ::std::task::Poll::Ready(x) =
                                       ::std::future::poll_with_tls_waker(unsafe
                                                                          {
                                                                              ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                          }) {
                                    break x ;
                                }
                                yield
                            }
                        }?;
                    Ok(Self{channel, exchange, store_namespace, uri,})
                }
                /// Subscribe to an event
                pub async fn subscribe<ED>(&self, store: Store,
                                           options: SubscribeOptions)
                 -> Result<(), io::Error> where ED: EventData + EventHandler +
                 Debug + Send {
                    let channel =
                        {
                            let mut pinned =
                                amqp_connect(self.uri, &self.exchange);
                            loop  {
                                if let ::std::task::Poll::Ready(x) =
                                       ::std::future::poll_with_tls_waker(unsafe
                                                                          {
                                                                              ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                          }) {
                                    break x ;
                                }
                                yield
                            }
                        }?;
                    let event_namespace = ED::event_namespace();
                    let event_type = ED::event_type();
                    let event_name =
                        ::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["",
                                                                             "."],
                                                                           &match (&event_namespace,
                                                                                   &event_type)
                                                                                {
                                                                                (arg0,
                                                                                 arg1)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Display::fmt),
                                                                                 ::std::fmt::ArgumentV1::new(arg1,
                                                                                                             ::std::fmt::Display::fmt)],
                                                                            }));
                    let queue_name = self.namespaced_event_queue_name::<ED>();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                               lvl <= ::log::max_level() {
                            ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Subscribe queue "],
                                                                                   &match (&queue_name,)
                                                                                        {
                                                                                        (arg0,)
                                                                                        =>
                                                                                        [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                     ::std::fmt::Display::fmt)],
                                                                                    }),
                                                     lvl,
                                                     &("event_store::adapters::emitter::amqp",
                                                       "event_store::adapters::emitter::amqp",
                                                       "event-store/src/adapters/emitter/amqp.rs",
                                                       68u32));
                        }
                    };
                    let queue =
                        {
                            let mut pinned =
                                amqp_bind_queue(&channel, &queue_name,
                                                &self.exchange, &event_name);
                            loop  {
                                if let ::std::task::Poll::Ready(x) =
                                       ::std::future::poll_with_tls_waker(unsafe
                                                                          {
                                                                              ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                          }) {
                                    break x ;
                                }
                                yield
                            }
                        }.unwrap();
                    {
                        let lvl = ::log::Level::Info;
                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                               lvl <= ::log::max_level() {
                            ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Creating consumer for event ",
                                                                                     " on queue ",
                                                                                     " on exchange "],
                                                                                   &match (&event_name,
                                                                                           &queue_name,
                                                                                           &self.exchange)
                                                                                        {
                                                                                        (arg0,
                                                                                         arg1,
                                                                                         arg2)
                                                                                        =>
                                                                                        [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                     ::std::fmt::Display::fmt),
                                                                                         ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                     ::std::fmt::Display::fmt),
                                                                                         ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                     ::std::fmt::Display::fmt)],
                                                                                    }),
                                                     lvl,
                                                     &("event_store::adapters::emitter::amqp",
                                                       "event_store::adapters::emitter::amqp",
                                                       "event-store/src/adapters/emitter/amqp.rs",
                                                       78u32));
                        }
                    };
                    let mut stream: Consumer<TcpStream> =
                        {
                            let mut pinned =
                                forward(channel.basic_consume(&queue, &"",
                                                              BasicConsumeOptions::default(),
                                                              FieldTable::new()).map_err(|e|
                                                                                             io::Error::new(io::ErrorKind::Other,
                                                                                                            e.to_string())));
                            loop  {
                                if let ::std::task::Poll::Ready(x) =
                                       ::std::future::poll_with_tls_waker(unsafe
                                                                          {
                                                                              ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                          }) {
                                    break x ;
                                }
                                yield
                            }
                        }.unwrap();
                    tokio::spawn_async(async move
                                           {
                                               while let Some(Ok(message)) =
                                                         {
                                                             let mut pinned =
                                                                 stream.next();
                                                             loop  {
                                                                 if let ::std::task::Poll::Ready(x)
                                                                        =
                                                                        ::std::future::poll_with_tls_waker(unsafe
                                                                                                           {
                                                                                                               ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                                                           })
                                                                        {
                                                                     break x ;
                                                                 }
                                                                 yield
                                                             }
                                                         } {
                                                   let parsed =
                                                       serde_json::from_slice::<Event<ED>>(&message.data);
                                                   match parsed {
                                                       Ok(event) => {
                                                           {
                                                               let lvl =
                                                                   ::log::Level::Trace;
                                                               if lvl <=
                                                                      ::log::STATIC_MAX_LEVEL
                                                                      &&
                                                                      lvl <=
                                                                          ::log::max_level()
                                                                  {
                                                                   ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Received event "],
                                                                                                                          &match (&event.id,)
                                                                                                                               {
                                                                                                                               (arg0,)
                                                                                                                               =>
                                                                                                                               [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                            ::std::fmt::Display::fmt)],
                                                                                                                           }),
                                                                                            lvl,
                                                                                            &("event_store::adapters::emitter::amqp",
                                                                                              "event_store::adapters::emitter::amqp",
                                                                                              "event-store/src/adapters/emitter/amqp.rs",
                                                                                              103u32));
                                                               }
                                                           };
                                                           let saved =
                                                               if options.save_on_receive
                                                                  {
                                                                   {
                                                                       let lvl =
                                                                           ::log::Level::Trace;
                                                                       if lvl
                                                                              <=
                                                                              ::log::STATIC_MAX_LEVEL
                                                                              &&
                                                                              lvl
                                                                                  <=
                                                                                  ::log::max_level()
                                                                          {
                                                                           ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Save event ",
                                                                                                                                    " (",
                                                                                                                                    ".",
                                                                                                                                    ")"],
                                                                                                                                  &match (&event.id,
                                                                                                                                          &ED::event_namespace(),
                                                                                                                                          &ED::event_type())
                                                                                                                                       {
                                                                                                                                       (arg0,
                                                                                                                                        arg1,
                                                                                                                                        arg2)
                                                                                                                                       =>
                                                                                                                                       [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                    ::std::fmt::Display::fmt),
                                                                                                                                        ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                    ::std::fmt::Display::fmt),
                                                                                                                                        ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                    ::std::fmt::Display::fmt)],
                                                                                                                                   }),
                                                                                                    lvl,
                                                                                                    &("event_store::adapters::emitter::amqp",
                                                                                                      "event_store::adapters::emitter::amqp",
                                                                                                      "event-store/src/adapters/emitter/amqp.rs",
                                                                                                      106u32));
                                                                       }
                                                                   };
                                                                   store.save_no_emit(&event)
                                                               } else {
                                                                   {
                                                                       let lvl =
                                                                           ::log::Level::Trace;
                                                                       if lvl
                                                                              <=
                                                                              ::log::STATIC_MAX_LEVEL
                                                                              &&
                                                                              lvl
                                                                                  <=
                                                                                  ::log::max_level()
                                                                          {
                                                                           ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Skip saving event ",
                                                                                                                                    " (",
                                                                                                                                    ".",
                                                                                                                                    ")"],
                                                                                                                                  &match (&event.id,
                                                                                                                                          &ED::event_namespace(),
                                                                                                                                          &ED::event_type())
                                                                                                                                       {
                                                                                                                                       (arg0,
                                                                                                                                        arg1,
                                                                                                                                        arg2)
                                                                                                                                       =>
                                                                                                                                       [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                    ::std::fmt::Display::fmt),
                                                                                                                                        ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                    ::std::fmt::Display::fmt),
                                                                                                                                        ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                    ::std::fmt::Display::fmt)],
                                                                                                                                   }),
                                                                                                    lvl,
                                                                                                    &("event_store::adapters::emitter::amqp",
                                                                                                      "event_store::adapters::emitter::amqp",
                                                                                                      "event-store/src/adapters/emitter/amqp.rs",
                                                                                                      115u32));
                                                                       }
                                                                   };
                                                                   Ok(SaveStatus::Ok)
                                                               };
                                                           saved.map(|result|
                                                                         match result
                                                                             {
                                                                             SaveStatus::Ok
                                                                             =>
                                                                             {
                                                                                 {
                                                                                     let lvl =
                                                                                         ::log::Level::Trace;
                                                                                     if lvl
                                                                                            <=
                                                                                            ::log::STATIC_MAX_LEVEL
                                                                                            &&
                                                                                            lvl
                                                                                                <=
                                                                                                ::log::max_level()
                                                                                        {
                                                                                         ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Event saved, calling handler"],
                                                                                                                                                &match ()
                                                                                                                                                     {
                                                                                                                                                     ()
                                                                                                                                                     =>
                                                                                                                                                     [],
                                                                                                                                                 }),
                                                                                                                  lvl,
                                                                                                                  &("event_store::adapters::emitter::amqp",
                                                                                                                    "event_store::adapters::emitter::amqp",
                                                                                                                    "event-store/src/adapters/emitter/amqp.rs",
                                                                                                                    129u32));
                                                                                     }
                                                                                 };
                                                                                 ED::handle_event(event,
                                                                                                  &store);
                                                                             }
                                                                             SaveStatus::Duplicate
                                                                             =>
                                                                             {
                                                                                 {
                                                                                     let lvl =
                                                                                         ::log::Level::Debug;
                                                                                     if lvl
                                                                                            <=
                                                                                            ::log::STATIC_MAX_LEVEL
                                                                                            &&
                                                                                            lvl
                                                                                                <=
                                                                                                ::log::max_level()
                                                                                        {
                                                                                         ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Duplicate event ",
                                                                                                                                                  ", skipping handler"],
                                                                                                                                                &match (&event.id,)
                                                                                                                                                     {
                                                                                                                                                     (arg0,)
                                                                                                                                                     =>
                                                                                                                                                     [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                  ::std::fmt::Display::fmt)],
                                                                                                                                                 }),
                                                                                                                  lvl,
                                                                                                                  &("event_store::adapters::emitter::amqp",
                                                                                                                    "event_store::adapters::emitter::amqp",
                                                                                                                    "event-store/src/adapters/emitter/amqp.rs",
                                                                                                                    133u32));
                                                                                     }
                                                                                 };
                                                                             }
                                                                         }).expect("Failed to handle event");
                                                           {
                                                               let lvl =
                                                                   ::log::Level::Trace;
                                                               if lvl <=
                                                                      ::log::STATIC_MAX_LEVEL
                                                                      &&
                                                                      lvl <=
                                                                          ::log::max_level()
                                                                  {
                                                                   ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Ack event "],
                                                                                                                          &match (&message.delivery_tag,)
                                                                                                                               {
                                                                                                                               (arg0,)
                                                                                                                               =>
                                                                                                                               [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                            ::std::fmt::Display::fmt)],
                                                                                                                           }),
                                                                                            lvl,
                                                                                            &("event_store::adapters::emitter::amqp",
                                                                                              "event_store::adapters::emitter::amqp",
                                                                                              "event-store/src/adapters/emitter/amqp.rs",
                                                                                              138u32));
                                                               }
                                                           };
                                                           {
                                                               let mut pinned =
                                                                   forward(channel.basic_ack(message.delivery_tag,
                                                                                             false));
                                                               loop  {
                                                                   if let ::std::task::Poll::Ready(x)
                                                                          =
                                                                          ::std::future::poll_with_tls_waker(unsafe
                                                                                                             {
                                                                                                                 ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                                                             })
                                                                          {
                                                                       break x
                                                                           ;
                                                                   }
                                                                   yield
                                                               }
                                                           }.expect("Could not ack message");
                                                       }
                                                       Err(e) => {
                                                           {
                                                               let lvl =
                                                                   ::log::Level::Trace;
                                                               if lvl <=
                                                                      ::log::STATIC_MAX_LEVEL
                                                                      &&
                                                                      lvl <=
                                                                          ::log::max_level()
                                                                  {
                                                                   ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Failed event payload: "],
                                                                                                                          &match (&String::from_utf8(message.data.clone()).unwrap_or(String::from("(failed to decode message)")),)
                                                                                                                               {
                                                                                                                               (arg0,)
                                                                                                                               =>
                                                                                                                               [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                            ::std::fmt::Display::fmt)],
                                                                                                                           }),
                                                                                            lvl,
                                                                                            &("event_store::adapters::emitter::amqp",
                                                                                              "event_store::adapters::emitter::amqp",
                                                                                              "event-store/src/adapters/emitter/amqp.rs",
                                                                                              144u32));
                                                               }
                                                           };
                                                           serde_json::from_slice::<JsonValue>(&message.data).map(|evt|
                                                                                                                      {
                                                                                                                          {
                                                                                                                              let lvl =
                                                                                                                                  ::log::Level::Error;
                                                                                                                              if lvl
                                                                                                                                     <=
                                                                                                                                     ::log::STATIC_MAX_LEVEL
                                                                                                                                     &&
                                                                                                                                     lvl
                                                                                                                                         <=
                                                                                                                                         ::log::max_level()
                                                                                                                                 {
                                                                                                                                  ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Failed to parse event ",
                                                                                                                                                                                           " (ID ",
                                                                                                                                                                                           "): "],
                                                                                                                                                                                         &match (&ED::event_namespace_and_type(),
                                                                                                                                                                                                 &evt["id"],
                                                                                                                                                                                                 &e.to_string())
                                                                                                                                                                                              {
                                                                                                                                                                                              (arg0,
                                                                                                                                                                                               arg1,
                                                                                                                                                                                               arg2)
                                                                                                                                                                                              =>
                                                                                                                                                                                              [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                                           ::std::fmt::Display::fmt),
                                                                                                                                                                                               ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                                           ::std::fmt::Display::fmt),
                                                                                                                                                                                               ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                                                                           ::std::fmt::Display::fmt)],
                                                                                                                                                                                          }),
                                                                                                                                                           lvl,
                                                                                                                                                           &("event_store::adapters::emitter::amqp",
                                                                                                                                                             "event_store::adapters::emitter::amqp",
                                                                                                                                                             "event-store/src/adapters/emitter/amqp.rs",
                                                                                                                                                             152u32));
                                                                                                                              }
                                                                                                                          };
                                                                                                                      }).unwrap_or_else(|_|
                                                                                                                                            {
                                                                                                                                                {
                                                                                                                                                    let lvl =
                                                                                                                                                        ::log::Level::Error;
                                                                                                                                                    if lvl
                                                                                                                                                           <=
                                                                                                                                                           ::log::STATIC_MAX_LEVEL
                                                                                                                                                           &&
                                                                                                                                                           lvl
                                                                                                                                                               <=
                                                                                                                                                               ::log::max_level()
                                                                                                                                                       {
                                                                                                                                                        ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Failed to parse event ",
                                                                                                                                                                                                                 " (ID unknown): "],
                                                                                                                                                                                                               &match (&ED::event_namespace_and_type(),
                                                                                                                                                                                                                       &e.to_string())
                                                                                                                                                                                                                    {
                                                                                                                                                                                                                    (arg0,
                                                                                                                                                                                                                     arg1)
                                                                                                                                                                                                                    =>
                                                                                                                                                                                                                    [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                                                                 ::std::fmt::Display::fmt),
                                                                                                                                                                                                                     ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                                                                 ::std::fmt::Display::fmt)],
                                                                                                                                                                                                                }),
                                                                                                                                                                                 lvl,
                                                                                                                                                                                 &("event_store::adapters::emitter::amqp",
                                                                                                                                                                                   "event_store::adapters::emitter::amqp",
                                                                                                                                                                                   "event-store/src/adapters/emitter/amqp.rs",
                                                                                                                                                                                   160u32));
                                                                                                                                                    }
                                                                                                                                                };
                                                                                                                                            });
                                                       }
                                                   }
                                               }
                                           });
                    Ok(())
                }
                /// Emit an event
                pub async fn emit<'a, ED>(&'a self, event: &'a Event<ED>)
                 -> Result<(), io::Error> where ED: EventData {
                    let payload: Vec<u8> =
                        serde_json::to_string(&event).expect("Cant serialise event").into();
                    let event_namespace = ED::event_namespace();
                    let event_type = ED::event_type();
                    let event_name =
                        ::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["",
                                                                             "."],
                                                                           &match (&event_namespace,
                                                                                   &event_type)
                                                                                {
                                                                                (arg0,
                                                                                 arg1)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Display::fmt),
                                                                                 ::std::fmt::ArgumentV1::new(arg1,
                                                                                                             ::std::fmt::Display::fmt)],
                                                                            }));
                    let queue_name = self.namespaced_event_queue_name::<ED>();
                    {
                        let lvl = ::log::Level::Info;
                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                               lvl <= ::log::max_level() {
                            ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Emitting event ",
                                                                                     " onto exchange ",
                                                                                     " through queue "],
                                                                                   &match (&event_name,
                                                                                           &self.exchange,
                                                                                           &queue_name)
                                                                                        {
                                                                                        (arg0,
                                                                                         arg1,
                                                                                         arg2)
                                                                                        =>
                                                                                        [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                     ::std::fmt::Display::fmt),
                                                                                         ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                     ::std::fmt::Display::fmt),
                                                                                         ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                     ::std::fmt::Display::fmt)],
                                                                                    }),
                                                     lvl,
                                                     &("event_store::adapters::emitter::amqp",
                                                       "event_store::adapters::emitter::amqp",
                                                       "event-store/src/adapters/emitter/amqp.rs",
                                                       189u32));
                        }
                    };
                    {
                        let mut pinned =
                            amqp_emit_data(&self.channel, &self.exchange,
                                           &event_name, payload);
                        loop  {
                            if let ::std::task::Poll::Ready(x) =
                                   ::std::future::poll_with_tls_waker(unsafe {
                                                                          ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                      }) {
                                break x ;
                            }
                            yield
                        }
                    }?;
                    Ok(())
                }
                pub(crate) async fn emit_value<'a,
                                               V>(&'a self,
                                                  event_namespace: &'a str,
                                                  event_type: &'a str,
                                                  data: &'a V)
                 -> Result<(), io::Error> where V: Serialize {
                    let payload: Vec<u8> =
                        serde_json::to_string(&data).expect("Cant serialise data").into();
                    let event_name =
                        ::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["",
                                                                             "."],
                                                                           &match (&event_namespace,
                                                                                   &event_type)
                                                                                {
                                                                                (arg0,
                                                                                 arg1)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Display::fmt),
                                                                                 ::std::fmt::ArgumentV1::new(arg1,
                                                                                                             ::std::fmt::Display::fmt)],
                                                                            }));
                    let queue_name =
                        ::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["",
                                                                             "-"],
                                                                           &match (&self.store_namespace,
                                                                                   &event_name)
                                                                                {
                                                                                (arg0,
                                                                                 arg1)
                                                                                =>
                                                                                [::std::fmt::ArgumentV1::new(arg0,
                                                                                                             ::std::fmt::Display::fmt),
                                                                                 ::std::fmt::ArgumentV1::new(arg1,
                                                                                                             ::std::fmt::Display::fmt)],
                                                                            }));
                    {
                        let lvl = ::log::Level::Info;
                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                               lvl <= ::log::max_level() {
                            ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Emitting data ",
                                                                                     " onto exchange ",
                                                                                     " through queue "],
                                                                                   &match (&event_name,
                                                                                           &self.exchange,
                                                                                           &queue_name)
                                                                                        {
                                                                                        (arg0,
                                                                                         arg1,
                                                                                         arg2)
                                                                                        =>
                                                                                        [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                     ::std::fmt::Display::fmt),
                                                                                         ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                     ::std::fmt::Display::fmt),
                                                                                         ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                     ::std::fmt::Display::fmt)],
                                                                                    }),
                                                     lvl,
                                                     &("event_store::adapters::emitter::amqp",
                                                       "event_store::adapters::emitter::amqp",
                                                       "event-store/src/adapters/emitter/amqp.rs",
                                                       220u32));
                        }
                    };
                    {
                        let mut pinned =
                            amqp_emit_data(&self.channel, &self.exchange,
                                           &event_name, payload);
                        loop  {
                            if let ::std::task::Poll::Ready(x) =
                                   ::std::future::poll_with_tls_waker(unsafe {
                                                                          ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                      }) {
                                break x ;
                            }
                            yield
                        }
                    }?;
                    Ok(())
                }
                fn namespaced_event_queue_name<ED>(&self) -> String where
                 ED: EventData {
                    ::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["",
                                                                         "-"],
                                                                       &match (&self.store_namespace,
                                                                               &self.event_queue_name::<ED>())
                                                                            {
                                                                            (arg0,
                                                                             arg1)
                                                                            =>
                                                                            [::std::fmt::ArgumentV1::new(arg0,
                                                                                                         ::std::fmt::Display::fmt),
                                                                             ::std::fmt::ArgumentV1::new(arg1,
                                                                                                         ::std::fmt::Display::fmt)],
                                                                        }))
                }
                fn event_queue_name<ED>(&self) -> String where ED: EventData {
                    ::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["",
                                                                         "."],
                                                                       &match (&ED::event_namespace(),
                                                                               &ED::event_type())
                                                                            {
                                                                            (arg0,
                                                                             arg1)
                                                                            =>
                                                                            [::std::fmt::ArgumentV1::new(arg0,
                                                                                                         ::std::fmt::Display::fmt),
                                                                             ::std::fmt::ArgumentV1::new(arg1,
                                                                                                         ::std::fmt::Display::fmt)],
                                                                        }))
                }
            }
            async fn amqp_connect(uri: SocketAddr, exchange: &String)
             -> Result<Channel<TcpStream>, io::Error> {
                let exchange1 = exchange.clone();
                let stream: TcpStream =
                    {
                        let mut pinned = forward(TcpStream::connect(&uri));
                        loop  {
                            if let ::std::task::Poll::Ready(x) =
                                   ::std::future::poll_with_tls_waker(unsafe {
                                                                          ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                      }) {
                                break x ;
                            }
                            yield
                        }
                    }?;
                let (client, heartbeat) =
                    {
                        let mut pinned =
                            forward(Client::connect(stream,
                                                    ConnectionOptions{frame_max:
                                                                          65535,
                                                                      heartbeat:
                                                                          120,
                                                                                 ..ConnectionOptions::default()}));
                        loop  {
                            if let ::std::task::Poll::Ready(x) =
                                   ::std::future::poll_with_tls_waker(unsafe {
                                                                          ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                      }) {
                                break x ;
                            }
                            yield
                        }
                    }.map_err(|e|
                                  io::Error::new(io::ErrorKind::Other,
                                                 e.to_string()))?;
                tokio::spawn(heartbeat.map_err(|e|
                                                   {
                                                       ::std::io::_eprint(::std::fmt::Arguments::new_v1(&["heartbeat error: ",
                                                                                                          "\n"],
                                                                                                        &match (&e,)
                                                                                                             {
                                                                                                             (arg0,)
                                                                                                             =>
                                                                                                             [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                          ::std::fmt::Debug::fmt)],
                                                                                                         }));
                                                   }));
                let channel =
                    {
                        let mut pinned = forward(client.create_channel());
                        loop  {
                            if let ::std::task::Poll::Ready(x) =
                                   ::std::future::poll_with_tls_waker(unsafe {
                                                                          ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                      }) {
                                break x ;
                            }
                            yield
                        }
                    }.map_err(|e|
                                  io::Error::new(io::ErrorKind::Other,
                                                 e.to_string()))?;
                {
                    let mut pinned =
                        forward(channel.exchange_declare(&exchange1, &"topic",
                                                         ExchangeDeclareOptions{durable:
                                                                                    true,
                                                                                            ..ExchangeDeclareOptions::default()},
                                                         FieldTable::new()));
                    loop  {
                        if let ::std::task::Poll::Ready(x) =
                               ::std::future::poll_with_tls_waker(unsafe {
                                                                      ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                  }) {
                            break x ;
                        }
                        yield
                    }
                }.map_err(|e|
                              io::Error::new(io::ErrorKind::Other,
                                             e.to_string()))?;
                Ok(channel)
            }
            async fn amqp_bind_queue<'a>(channel: &'a Channel<TcpStream>,
                                         queue_name: &'a String,
                                         exchange_name: &'a String,
                                         routing_key: &'a String)
             -> Result<Queue, io::Error> {
                {
                    let lvl = ::log::Level::Debug;
                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                           lvl <= ::log::max_level() {
                        ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Bind queue ",
                                                                                 " to exchange ",
                                                                                 " through routing key "],
                                                                               &match (&queue_name,
                                                                                       &exchange_name,
                                                                                       &routing_key)
                                                                                    {
                                                                                    (arg0,
                                                                                     arg1,
                                                                                     arg2)
                                                                                    =>
                                                                                    [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                 ::std::fmt::Display::fmt),
                                                                                     ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                 ::std::fmt::Display::fmt),
                                                                                     ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                 ::std::fmt::Display::fmt)],
                                                                                }),
                                                 lvl,
                                                 &("event_store::adapters::emitter::amqp",
                                                   "event_store::adapters::emitter::amqp",
                                                   "event-store/src/adapters/emitter/amqp.rs",
                                                   290u32));
                    }
                };
                let queue =
                    {
                        let mut pinned =
                            forward(channel.queue_declare(&queue_name,
                                                          QueueDeclareOptions{durable:
                                                                                  true,
                                                                              exclusive:
                                                                                  false,
                                                                              auto_delete:
                                                                                  false,
                                                                                           ..QueueDeclareOptions::default()},
                                                          FieldTable::new()));
                        loop  {
                            if let ::std::task::Poll::Ready(x) =
                                   ::std::future::poll_with_tls_waker(unsafe {
                                                                          ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                      }) {
                                break x ;
                            }
                            yield
                        }
                    }.unwrap();
                {
                    let mut pinned =
                        forward(channel.queue_bind(&queue_name,
                                                   &exchange_name,
                                                   &routing_key,
                                                   QueueBindOptions::default(),
                                                   FieldTable::new()).map_err(|e|
                                                                                  io::Error::new(io::ErrorKind::Other,
                                                                                                 e.to_string())));
                    loop  {
                        if let ::std::task::Poll::Ready(x) =
                               ::std::future::poll_with_tls_waker(unsafe {
                                                                      ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                  }) {
                            break x ;
                        }
                        yield
                    }
                }?;
                Ok(queue)
            }
            async fn amqp_emit_data<'a>(channel: &'a Channel<TcpStream>,
                                        exchange: &'a String,
                                        routing_key: &'a String,
                                        payload: Vec<u8>)
             -> Result<(), io::Error> {
                {
                    let lvl = ::log::Level::Debug;
                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                           lvl <= ::log::max_level() {
                        ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Emitting payload through routing key ",
                                                                                 " onto exchange "],
                                                                               &match (&routing_key,
                                                                                       &exchange)
                                                                                    {
                                                                                    (arg0,
                                                                                     arg1)
                                                                                    =>
                                                                                    [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                 ::std::fmt::Display::fmt),
                                                                                     ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                 ::std::fmt::Display::fmt)],
                                                                                }),
                                                 lvl,
                                                 &("event_store::adapters::emitter::amqp",
                                                   "event_store::adapters::emitter::amqp",
                                                   "event-store/src/adapters/emitter/amqp.rs",
                                                   328u32));
                    }
                };
                {
                    let mut pinned =
                        forward(channel.basic_publish(&exchange, &routing_key,
                                                      payload,
                                                      BasicPublishOptions::default(),
                                                      BasicProperties::default()).map_err(|e|
                                                                                              io::Error::new(io::ErrorKind::Other,
                                                                                                             e.to_string())));
                    loop  {
                        if let ::std::task::Poll::Ready(x) =
                               ::std::future::poll_with_tls_waker(unsafe {
                                                                      ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                  }) {
                            break x ;
                        }
                        yield
                    }
                }?;
                Ok(())
            }
        }
        pub use self::amqp::AmqpEmitterAdapter;
    }
    mod store {
        mod pg {
            use crate::event::Event;
            use crate::event_context::EventContext;
            use crate::store_query::StoreQuery;
            use chrono::prelude::*;
            use event_store_derive_internals::EventData;
            use event_store_derive_internals::Events;
            use fallible_iterator::FallibleIterator;
            use log::{debug, trace};
            use postgres::error::UNIQUE_VIOLATION;
            use r2d2::Pool;
            use r2d2_postgres::postgres::types::ToSql;
            use r2d2_postgres::PostgresConnectionManager;
            use serde_json::{from_value, json, to_value, Value as JsonValue};
            use sha2::{Digest, Sha256};
            use std::io;
            use uuid::Uuid;
            const INIT_QUERIES: &'static str =
                r#"
-- Create UUID extension just in case
create extension if not exists "uuid-ossp";

-- Create events table if it doesn't already exist
create table if not exists events(
    id uuid default uuid_generate_v4() primary key,
    data jsonb not null,
    context jsonb default '{}'
);

-- Add index on sequence number and time to speed up ordering
create index if not exists counter_time on events ((context->>'time') asc);

-- Create index to speed up queries by type
create index if not exists event_type_legacy on events ((data->>'type') nulls last);
create index if not exists event_namespace_and_type on events ((context->>'event_namespace') nulls last, (context->>'event_type') nulls last);
"#;
            /// Representation of a Postgres query and args
            pub struct PgQuery {
                /// Query string with placeholders
                pub query: String,
                /// Arguments to use for the query
                pub args: Vec<Box<ToSql + Send + Sync>>,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::fmt::Debug for PgQuery {
                fn fmt(&self, f: &mut ::std::fmt::Formatter)
                 -> ::std::fmt::Result {
                    match *self {
                        PgQuery { query: ref __self_0_0, args: ref __self_0_1
                        } => {
                            let mut debug_trait_builder =
                                f.debug_struct("PgQuery");
                            let _ =
                                debug_trait_builder.field("query",
                                                          &&(*__self_0_0));
                            let _ =
                                debug_trait_builder.field("args",
                                                          &&(*__self_0_1));
                            debug_trait_builder.finish()
                        }
                    }
                }
            }
            impl PgQuery {
                /// Create a new query from a query string and arguments
                pub fn new(query: &str, args: Vec<Box<ToSql + Send + Sync>>)
                 -> Self {
                    Self{query: query.into(), args,}
                }
            }
            impl StoreQuery for PgQuery {
                fn unique_id(&self) -> String {
                    let hash =
                        Sha256::digest(::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["",
                                                                                            ":[",
                                                                                            "]"],
                                                                                          &match (&self.args,
                                                                                                  &self.query)
                                                                                               {
                                                                                               (arg0,
                                                                                                arg1)
                                                                                               =>
                                                                                               [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                            ::std::fmt::Debug::fmt),
                                                                                                ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                            ::std::fmt::Display::fmt)],
                                                                                           })).as_bytes());
                    hash.iter().fold(String::new(),
                                     |mut acc, hex|
                                         {
                                             acc.push_str(&::alloc::fmt::format(::std::fmt::Arguments::new_v1(&[""],
                                                                                                              &match (&hex,)
                                                                                                                   {
                                                                                                                   (arg0,)
                                                                                                                   =>
                                                                                                                   [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                ::std::fmt::UpperHex::fmt)],
                                                                                                               })));
                                             acc
                                         })
                }
            }
            fn generate_query(initial_query: &PgQuery,
                              since: Option<DateTime<Utc>>) -> String {
                if let Some(timestamp) = since {
                    String::from(::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["select * from (",
                                                                                      ") as events where (events.context->>\'time\')::timestamp with time zone >= \'",
                                                                                      "\' order by (events.context->>\'time\')::timestamp with time zone asc"],
                                                                                    &match (&initial_query.query,
                                                                                            &timestamp)
                                                                                         {
                                                                                         (arg0,
                                                                                          arg1)
                                                                                         =>
                                                                                         [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                      ::std::fmt::Display::fmt),
                                                                                          ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                      ::std::fmt::Display::fmt)],
                                                                                     })))
                } else {
                    String::from(::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["select * from (",
                                                                                      ") as events order by (events.context->>\'time\')::timestamp with time zone asc"],
                                                                                    &match (&initial_query.query,)
                                                                                         {
                                                                                         (arg0,)
                                                                                         =>
                                                                                         [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                      ::std::fmt::Display::fmt)],
                                                                                     })))
                }
            }
            /// Save result
            pub enum SaveStatus {

                /// The save was successful
                Ok,

                /// A duplicate item already exists in the backing store
                Duplicate,
            }
            /// The result of a save operation
            ///
            /// If the save did not error but a duplicate was encountered, this should be equal to
            /// `Ok(SaveStatus::Ok)`
            pub type SaveResult = Result<SaveStatus, io::Error>;
            /// Postgres-backed store adapter
            pub struct PgStoreAdapter {
                conn: Pool<PostgresConnectionManager>,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::std::clone::Clone for PgStoreAdapter {
                #[inline]
                fn clone(&self) -> PgStoreAdapter {
                    match *self {
                        PgStoreAdapter { conn: ref __self_0_0 } =>
                        PgStoreAdapter{conn:
                                           ::std::clone::Clone::clone(&(*__self_0_0)),},
                    }
                }
            }
            impl PgStoreAdapter {
                /// Create a new Postgres store
                ///
                /// This will attempt to create the events table and indexes if they do not already exist
                pub async fn new(conn: Pool<PostgresConnectionManager>)
                 -> Result<Self, io::Error> {
                    conn.get().map_err(|e|
                                           io::Error::new(io::ErrorKind::Other,
                                                          e.to_string()))?.batch_execute(INIT_QUERIES)?;
                    Ok(Self{conn,})
                }
                /// Save an event into PG
                pub fn save<'a, ED>(&'a self, event: &'a Event<ED>)
                 -> SaveResult where ED: EventData {
                    {
                        let lvl = ::log::Level::Debug;
                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                               lvl <= ::log::max_level() {
                            ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Insert event ",
                                                                                     "."],
                                                                                   &match (&ED::event_namespace(),
                                                                                           &ED::event_type())
                                                                                        {
                                                                                        (arg0,
                                                                                         arg1)
                                                                                        =>
                                                                                        [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                     ::std::fmt::Display::fmt),
                                                                                         ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                     ::std::fmt::Display::fmt)],
                                                                                    }),
                                                     lvl,
                                                     &("event_store::adapters::store::pg",
                                                       "event_store::adapters::store::pg",
                                                       "event-store/src/adapters/store/pg.rs",
                                                       119u32));
                        }
                    };
                    self.conn.get().unwrap().execute("insert into events (id, data, context) values ($1, $2, $3)",
                                                     &[&event.id,
                                                       &to_value(&event.data).expect("Unable to convert event data to value"),
                                                       &to_value(&event.context).expect("Cannot convert event context")]).map(|_|
                                                                                                                                  Ok(SaveStatus::Ok)).unwrap_or_else(|err|
                                                                                                                                                                         {
                                                                                                                                                                             let is_duplicate_error =
                                                                                                                                                                                 err.code().unwrap()
                                                                                                                                                                                     ==
                                                                                                                                                                                     &UNIQUE_VIOLATION;
                                                                                                                                                                             if is_duplicate_error
                                                                                                                                                                                {
                                                                                                                                                                                 Ok(SaveStatus::Duplicate)
                                                                                                                                                                             } else {
                                                                                                                                                                                 Err(io::Error::new(io::ErrorKind::Other,
                                                                                                                                                                                                    ::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["Could not save event: "],
                                                                                                                                                                                                                                                       &match (&err,)
                                                                                                                                                                                                                                                            {
                                                                                                                                                                                                                                                            (arg0,)
                                                                                                                                                                                                                                                            =>
                                                                                                                                                                                                                                                            [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                                                                                                         ::std::fmt::Display::fmt)],
                                                                                                                                                                                                                                                        }))))
                                                                                                                                                                             }
                                                                                                                                                                         })
                }
                /// Read a list of events
                pub async fn read<'a,
                                  E>(&'a self, query: &'a PgQuery,
                                     since: Option<DateTime<Utc>>)
                 -> Result<Vec<E>, io::Error> where E: Events {
                    let query_string = generate_query(&query, since);
                    {
                        let lvl = ::log::Level::Debug;
                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                               lvl <= ::log::max_level() {
                            ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Read query "],
                                                                                   &match (&query_string,)
                                                                                        {
                                                                                        (arg0,)
                                                                                        =>
                                                                                        [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                     ::std::fmt::Display::fmt)],
                                                                                    }),
                                                     lvl,
                                                     &("event_store::adapters::store::pg",
                                                       "event_store::adapters::store::pg",
                                                       "event-store/src/adapters/store/pg.rs",
                                                       162u32));
                        }
                    };
                    let conn = self.conn.get().unwrap();
                    let trans =
                        conn.transaction().expect("Unable to initialise transaction");
                    let stmt =
                        trans.prepare(&query_string).expect("Unable to prepare read statement");
                    let mut params: Vec<&ToSql> = Vec::new();
                    for (i, _arg) in query.args.iter().enumerate() {
                        params.push(&*query.args[i]);
                    }
                    let results =
                        stmt.lazy_query(&trans, &params,
                                        1000).unwrap().map(|row|
                                                               {
                                                                   let id:
                                                                           Uuid =
                                                                       row.get("id");
                                                                   let data_json:
                                                                           JsonValue =
                                                                       row.get("data");
                                                                   let context_json:
                                                                           JsonValue =
                                                                       row.get("context");
                                                                   let thing =
                                                                       ::serde_json::Value::Object({
                                                                                                       let mut object =
                                                                                                           ::serde_json::Map::new();
                                                                                                       let _ =
                                                                                                           object.insert(("id").into(),
                                                                                                                         ::serde_json::to_value(&id).unwrap());
                                                                                                       let _ =
                                                                                                           object.insert(("data").into(),
                                                                                                                         ::serde_json::to_value(&data_json).unwrap());
                                                                                                       let _ =
                                                                                                           object.insert(("context").into(),
                                                                                                                         ::serde_json::to_value(&context_json).unwrap());
                                                                                                       object
                                                                                                   });
                                                                   let evt:
                                                                           E =
                                                                       from_value(thing).expect("Could not decode row");
                                                                   evt
                                                               }).collect().expect("Failed to collect results");
                    trans.finish().expect("Could not finish transaction");
                    Ok(results)
                }
                /// Find the most recent event of a given type
                pub fn last_event<ED>(&self)
                 -> Result<Option<Event<ED>>, io::Error> where ED: EventData {
                    let rows =
                        self.conn.get().unwrap().query(r#"select * from events
                    where data->>'event_namespace' = $1
                    and data->>'event_type' = $2
                    order by (context->>'time')::timestamp with time zone desc
                    limit 1"#,
                                                       &[&ED::event_namespace(),
                                                         &ED::event_type()]).expect("Unable to query database (last_event)");
                    if rows.len() == 1 {
                        let row = rows.get(0);
                        let id: Uuid = row.get("id");
                        let data_json: JsonValue = row.get("data");
                        let context_json: JsonValue = row.get("context");
                        let data: ED = from_value(data_json).unwrap();
                        let context: EventContext =
                            from_value(context_json).unwrap();
                        Ok(Some(Event{id, data, context,}))
                    } else { Ok(None) }
                }
                /// Fetch events of a given type starting from a timestamp going forward
                pub async fn read_events_since<'a>(&'a self,
                                                   event_namespace: &'a str,
                                                   event_type: &'a str,
                                                   since: DateTime<Utc>)
                 -> Result<Vec<JsonValue>, io::Error> {
                    let query_string =
                        r#"select * from events
            where data->>'event_namespace' = $1
            and data->>'event_type' = $2
            and context->>'time' >= $3
            order by (context->>'time')::timestamp with time zone asc"#;
                    let conn = self.conn.get().unwrap();
                    let trans =
                        conn.transaction().expect("Unable to initialise transaction");
                    let stmt =
                        trans.prepare(&query_string).expect("Unable to prepare read statement");
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                               lvl <= ::log::max_level() {
                            ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["Read events of type ",
                                                                                     ".",
                                                                                     " since "],
                                                                                   &match (&event_namespace,
                                                                                           &event_type,
                                                                                           &since.to_rfc3339())
                                                                                        {
                                                                                        (arg0,
                                                                                         arg1,
                                                                                         arg2)
                                                                                        =>
                                                                                        [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                     ::std::fmt::Display::fmt),
                                                                                         ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                     ::std::fmt::Display::fmt),
                                                                                         ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                     ::std::fmt::Display::fmt)],
                                                                                    }),
                                                     lvl,
                                                     &("event_store::adapters::store::pg",
                                                       "event_store::adapters::store::pg",
                                                       "event-store/src/adapters/store/pg.rs",
                                                       263u32));
                        }
                    };
                    let results =
                        stmt.lazy_query(&trans,
                                        &[&event_namespace, &event_type,
                                          &since.to_rfc3339()],
                                        1000).map_err(|e|
                                                          io::Error::new(io::ErrorKind::Other,
                                                                         e.to_string()))?.map(|row|
                                                                                                  {
                                                                                                      let id:
                                                                                                              Uuid =
                                                                                                          row.get("id");
                                                                                                      let data_json:
                                                                                                              JsonValue =
                                                                                                          row.get("data");
                                                                                                      let context_json:
                                                                                                              JsonValue =
                                                                                                          row.get("context");
                                                                                                      ::serde_json::Value::Object({
                                                                                                                                      let mut object =
                                                                                                                                          ::serde_json::Map::new();
                                                                                                                                      let _ =
                                                                                                                                          object.insert(("id").into(),
                                                                                                                                                        ::serde_json::to_value(&id).unwrap());
                                                                                                                                      let _ =
                                                                                                                                          object.insert(("data").into(),
                                                                                                                                                        ::serde_json::to_value(&data_json).unwrap());
                                                                                                                                      let _ =
                                                                                                                                          object.insert(("context").into(),
                                                                                                                                                        ::serde_json::to_value(&context_json).unwrap());
                                                                                                                                      object
                                                                                                                                  })
                                                                                                  }).collect().expect("Failed to collect results");
                    trans.finish().expect("Could not finish transaction");
                    Ok(results)
                }
            }
        }
        pub use self::pg::{PgQuery, PgStoreAdapter, SaveResult, SaveStatus};
    }
    pub use self::cache::{CacheResult, PgCacheAdapter};
    pub use self::emitter::AmqpEmitterAdapter;
    pub use self::store::{PgQuery, PgStoreAdapter, SaveResult, SaveStatus};
}
#[doc(hidden)]
pub mod internals {
    pub mod test_helpers {
        use crate::adapters::PgQuery;
        use crate::aggregator::Aggregator;
        use crate::event::Event;
        use crate::event_handler::EventHandler;
        use crate::store::Store;
        use event_store_derive::*;
        use log::trace;
        use postgres::types::ToSql;
        use r2d2::Pool;
        use r2d2_postgres::{PostgresConnectionManager, TlsMode};
        use serde_derive::*;
        use std::time::{SystemTime, UNIX_EPOCH};
        struct Asshole<'a> {
            field: &'a str,
        }
        #[allow(non_upper_case_globals,
                unused_attributes,
                unused_qualifications)]
        const _IMPL_DESERIALIZE_FOR_Asshole: () =
            {
                #[allow(unknown_lints)]
                #[allow(rust_2018_idioms)]
                extern crate serde as _serde;
                #[allow(unused_macros)]
                macro_rules! try(( $ __expr : expr ) => {
                                 match $ __expr {
                                 _serde :: export :: Ok ( __val ) => __val ,
                                 _serde :: export :: Err ( __err ) => {
                                 return _serde :: export :: Err ( __err ) ; }
                                 } });
                #[automatically_derived]
                impl <'de: 'a, 'a> _serde::Deserialize<'de> for Asshole<'a> {
                    fn deserialize<__D>(__deserializer: __D)
                     -> _serde::export::Result<Self, __D::Error> where
                     __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        enum __Field { __field0, __ignore, }
                        struct __FieldVisitor;
                        impl <'de> _serde::de::Visitor<'de> for __FieldVisitor
                         {
                            type
                            Value
                            =
                            __Field;
                            fn expecting(&self,
                                         __formatter:
                                             &mut _serde::export::Formatter)
                             -> _serde::export::fmt::Result {
                                _serde::export::Formatter::write_str(__formatter,
                                                                     "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                             -> _serde::export::Result<Self::Value, __E> where
                             __E: _serde::de::Error {
                                match __value {
                                    0u64 =>
                                    _serde::export::Ok(__Field::__field0),
                                    _ =>
                                    _serde::export::Err(_serde::de::Error::invalid_value(_serde::de::Unexpected::Unsigned(__value),
                                                                                         &"field index 0 <= i < 1")),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                             -> _serde::export::Result<Self::Value, __E> where
                             __E: _serde::de::Error {
                                match __value {
                                    "field" =>
                                    _serde::export::Ok(__Field::__field0),
                                    _ => {
                                        _serde::export::Ok(__Field::__ignore)
                                    }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                             -> _serde::export::Result<Self::Value, __E> where
                             __E: _serde::de::Error {
                                match __value {
                                    b"field" =>
                                    _serde::export::Ok(__Field::__field0),
                                    _ => {
                                        _serde::export::Ok(__Field::__ignore)
                                    }
                                }
                            }
                        }
                        impl <'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                             -> _serde::export::Result<Self, __D::Error> where
                             __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                                                             __FieldVisitor)
                            }
                        }
                        struct __Visitor<'de: 'a, 'a> {
                            marker: _serde::export::PhantomData<Asshole<'a>>,
                            lifetime: _serde::export::PhantomData<&'de ()>,
                        }
                        impl <'de: 'a, 'a> _serde::de::Visitor<'de> for
                         __Visitor<'de, 'a> {
                            type
                            Value
                            =
                            Asshole<'a>;
                            fn expecting(&self,
                                         __formatter:
                                             &mut _serde::export::Formatter)
                             -> _serde::export::fmt::Result {
                                _serde::export::Formatter::write_str(__formatter,
                                                                     "struct Asshole")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                             ->
                                 _serde::export::Result<Self::Value,
                                                        __A::Error> where
                             __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match match _serde::de::SeqAccess::next_element::<&'a str>(&mut __seq)
                                              {
                                              _serde::export::Ok(__val) =>
                                              __val,
                                              _serde::export::Err(__err) => {
                                                  return _serde::export::Err(__err);
                                              }
                                          } {
                                        _serde::export::Some(__value) =>
                                        __value,
                                        _serde::export::None => {
                                            return _serde::export::Err(_serde::de::Error::invalid_length(0usize,
                                                                                                         &"struct Asshole with 1 element"));
                                        }
                                    };
                                _serde::export::Ok(Asshole{field: __field0,})
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                             ->
                                 _serde::export::Result<Self::Value,
                                                        __A::Error> where
                             __A: _serde::de::MapAccess<'de> {
                                let mut __field0:
                                        _serde::export::Option<&'a str> =
                                    _serde::export::None;
                                while let _serde::export::Some(__key) =
                                          match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                              {
                                              _serde::export::Ok(__val) =>
                                              __val,
                                              _serde::export::Err(__err) => {
                                                  return _serde::export::Err(__err);
                                              }
                                          } {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::export::Option::is_some(&__field0)
                                               {
                                                return _serde::export::Err(<__A::Error
                                                                               as
                                                                               _serde::de::Error>::duplicate_field("field"));
                                            }
                                            __field0 =
                                                _serde::export::Some(match _serde::de::MapAccess::next_value::<&'a str>(&mut __map)
                                                                         {
                                                                         _serde::export::Ok(__val)
                                                                         =>
                                                                         __val,
                                                                         _serde::export::Err(__err)
                                                                         => {
                                                                             return _serde::export::Err(__err);
                                                                         }
                                                                     });
                                        }
                                        _ => {
                                            let _ =
                                                match _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)
                                                    {
                                                    _serde::export::Ok(__val)
                                                    => __val,
                                                    _serde::export::Err(__err)
                                                    => {
                                                        return _serde::export::Err(__err);
                                                    }
                                                };
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::export::Some(__field0) =>
                                        __field0,
                                        _serde::export::None =>
                                        match _serde::private::de::missing_field("field")
                                            {
                                            _serde::export::Ok(__val) =>
                                            __val,
                                            _serde::export::Err(__err) => {
                                                return _serde::export::Err(__err);
                                            }
                                        },
                                    };
                                _serde::export::Ok(Asshole{field: __field0,})
                            }
                        }
                        const FIELDS: &'static [&'static str] = &["field"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                                                                 "Asshole",
                                                                 FIELDS,
                                                                 __Visitor{marker:
                                                                               _serde::export::PhantomData::<Asshole<'a>>,
                                                                           lifetime:
                                                                               _serde::export::PhantomData,})
                    }
                }
            };
        /// Compile test: can structs with lifetimes be used?
        #[allow(unused)]
        #[event_store(namespace = "test_lifetimes")]
        struct LifetimeStruct<'a> {
            field: &'a str,
        }
        #[allow(non_upper_case_globals, unused_attributes, unused_imports)]
        const _IMPL_EVENT_STORE_STRUCT_FOR_LifetimeStruct: () =
            {
                extern crate serde;
                extern crate serde_derive;
                extern crate event_store_derive_internals;
                use serde::ser;
                use serde::de::{Deserialize, Deserializer};
                use serde::ser::{Serialize, Serializer, SerializeMap};
                impl <'a> event_store_derive_internals::EventData for
                 LifetimeStruct<'a> {
                    fn event_namespace_and_type() -> &'static str {
                        "test_lifetimes.LifetimeStruct"
                    }
                    fn event_namespace() -> &'static str { "test_lifetimes" }
                    fn event_type() -> &'static str { "LifetimeStruct" }
                }
                impl <'a> Serialize for LifetimeStruct<'a> {
                    fn serialize<S>(&self, serializer: S)
                     -> Result<S::Ok, S::Error> where S: Serializer {
                        let mut map = serializer.serialize_map(Some(4usize))?;
                        for (k, v) in self {
                            map.serialize_entry("#field_idents", self.field)?;
                        }
                        map.serialize_entry("event_namespace_and_type",
                                            "test_lifetimes.LifetimeStruct")?;
                        map.serialize_entry("event_namespace",
                                            "test_lifetimes")?;
                        map.serialize_entry("event_type", "LifetimeStruct")?;
                        map.end()
                    }
                }
                impl <'de: 'a, 'a> serde::Deserialize<'de> for
                 LifetimeStruct<'a> {
                    fn deserialize<__D>(deserializer: __D)
                     -> serde::export::Result<Self, __D::Error> where
                     __D: serde::Deserializer<'de> {
                        use serde::de;
                        struct EventIdent {
                            event_type: String,
                            event_namespace: String,
                        }
                        #[allow(non_upper_case_globals,
                                unused_attributes,
                                unused_qualifications)]
                        const _IMPL_DESERIALIZE_FOR_EventIdent: () =
                            {
                                #[allow(unknown_lints)]
                                #[allow(rust_2018_idioms)]
                                extern crate serde as _serde;
                                #[allow(unused_macros)]
                                macro_rules! try(( $ __expr : expr ) => {
                                                 match $ __expr {
                                                 _serde :: export :: Ok (
                                                 __val ) => __val , _serde ::
                                                 export :: Err ( __err ) => {
                                                 return _serde :: export ::
                                                 Err ( __err ) ; } } });
                                #[automatically_derived]
                                impl <'de> _serde::Deserialize<'de> for
                                 EventIdent {
                                    fn deserialize<__D>(__deserializer: __D)
                                     ->
                                         _serde::export::Result<Self,
                                                                __D::Error>
                                     where __D: _serde::Deserializer<'de> {
                                        #[allow(non_camel_case_types)]
                                        enum __Field {
                                            __field0,
                                            __field1,
                                            __ignore,
                                        }
                                        struct __FieldVisitor;
                                        impl <'de> _serde::de::Visitor<'de>
                                         for __FieldVisitor {
                                            type
                                            Value
                                            =
                                            __Field;
                                            fn expecting(&self,
                                                         __formatter:
                                                             &mut _serde::export::Formatter)
                                             -> _serde::export::fmt::Result {
                                                _serde::export::Formatter::write_str(__formatter,
                                                                                     "field identifier")
                                            }
                                            fn visit_u64<__E>(self,
                                                              __value: u64)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    0u64 =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    1u64 =>
                                                    _serde::export::Ok(__Field::__field1),
                                                    _ =>
                                                    _serde::export::Err(_serde::de::Error::invalid_value(_serde::de::Unexpected::Unsigned(__value),
                                                                                                         &"field index 0 <= i < 2")),
                                                }
                                            }
                                            fn visit_str<__E>(self,
                                                              __value: &str)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    "event_type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    "event_namespace" =>
                                                    _serde::export::Ok(__Field::__field1),
                                                    _ => {
                                                        _serde::export::Ok(__Field::__ignore)
                                                    }
                                                }
                                            }
                                            fn visit_bytes<__E>(self,
                                                                __value:
                                                                    &[u8])
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    b"event_type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    b"event_namespace" =>
                                                    _serde::export::Ok(__Field::__field1),
                                                    _ => {
                                                        _serde::export::Ok(__Field::__ignore)
                                                    }
                                                }
                                            }
                                        }
                                        impl <'de> _serde::Deserialize<'de>
                                         for __Field {
                                            #[inline]
                                            fn deserialize<__D>(__deserializer:
                                                                    __D)
                                             ->
                                                 _serde::export::Result<Self,
                                                                        __D::Error>
                                             where
                                             __D: _serde::Deserializer<'de> {
                                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                                                                             __FieldVisitor)
                                            }
                                        }
                                        struct __Visitor<'de> {
                                            marker: _serde::export::PhantomData<EventIdent>,
                                            lifetime: _serde::export::PhantomData<&'de ()>,
                                        }
                                        impl <'de> _serde::de::Visitor<'de>
                                         for __Visitor<'de> {
                                            type
                                            Value
                                            =
                                            EventIdent;
                                            fn expecting(&self,
                                                         __formatter:
                                                             &mut _serde::export::Formatter)
                                             -> _serde::export::fmt::Result {
                                                _serde::export::Formatter::write_str(__formatter,
                                                                                     "struct EventIdent")
                                            }
                                            #[inline]
                                            fn visit_seq<__A>(self,
                                                              mut __seq: __A)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __A::Error>
                                             where
                                             __A: _serde::de::SeqAccess<'de> {
                                                let __field0 =
                                                    match match _serde::de::SeqAccess::next_element::<String>(&mut __seq)
                                                              {
                                                              _serde::export::Ok(__val)
                                                              => __val,
                                                              _serde::export::Err(__err)
                                                              => {
                                                                  return _serde::export::Err(__err);
                                                              }
                                                          } {
                                                        _serde::export::Some(__value)
                                                        => __value,
                                                        _serde::export::None
                                                        => {
                                                            return _serde::export::Err(_serde::de::Error::invalid_length(0usize,
                                                                                                                         &"struct EventIdent with 2 elements"));
                                                        }
                                                    };
                                                let __field1 =
                                                    match match _serde::de::SeqAccess::next_element::<String>(&mut __seq)
                                                              {
                                                              _serde::export::Ok(__val)
                                                              => __val,
                                                              _serde::export::Err(__err)
                                                              => {
                                                                  return _serde::export::Err(__err);
                                                              }
                                                          } {
                                                        _serde::export::Some(__value)
                                                        => __value,
                                                        _serde::export::None
                                                        => {
                                                            return _serde::export::Err(_serde::de::Error::invalid_length(1usize,
                                                                                                                         &"struct EventIdent with 2 elements"));
                                                        }
                                                    };
                                                _serde::export::Ok(EventIdent{event_type:
                                                                                  __field0,
                                                                              event_namespace:
                                                                                  __field1,})
                                            }
                                            #[inline]
                                            fn visit_map<__A>(self,
                                                              mut __map: __A)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __A::Error>
                                             where
                                             __A: _serde::de::MapAccess<'de> {
                                                let mut __field0:
                                                        _serde::export::Option<String> =
                                                    _serde::export::None;
                                                let mut __field1:
                                                        _serde::export::Option<String> =
                                                    _serde::export::None;
                                                while let _serde::export::Some(__key)
                                                          =
                                                          match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                                              {
                                                              _serde::export::Ok(__val)
                                                              => __val,
                                                              _serde::export::Err(__err)
                                                              => {
                                                                  return _serde::export::Err(__err);
                                                              }
                                                          } {
                                                    match __key {
                                                        __Field::__field0 => {
                                                            if _serde::export::Option::is_some(&__field0)
                                                               {
                                                                return _serde::export::Err(<__A::Error
                                                                                               as
                                                                                               _serde::de::Error>::duplicate_field("event_type"));
                                                            }
                                                            __field0 =
                                                                _serde::export::Some(match _serde::de::MapAccess::next_value::<String>(&mut __map)
                                                                                         {
                                                                                         _serde::export::Ok(__val)
                                                                                         =>
                                                                                         __val,
                                                                                         _serde::export::Err(__err)
                                                                                         =>
                                                                                         {
                                                                                             return _serde::export::Err(__err);
                                                                                         }
                                                                                     });
                                                        }
                                                        __Field::__field1 => {
                                                            if _serde::export::Option::is_some(&__field1)
                                                               {
                                                                return _serde::export::Err(<__A::Error
                                                                                               as
                                                                                               _serde::de::Error>::duplicate_field("event_namespace"));
                                                            }
                                                            __field1 =
                                                                _serde::export::Some(match _serde::de::MapAccess::next_value::<String>(&mut __map)
                                                                                         {
                                                                                         _serde::export::Ok(__val)
                                                                                         =>
                                                                                         __val,
                                                                                         _serde::export::Err(__err)
                                                                                         =>
                                                                                         {
                                                                                             return _serde::export::Err(__err);
                                                                                         }
                                                                                     });
                                                        }
                                                        _ => {
                                                            let _ =
                                                                match _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)
                                                                    {
                                                                    _serde::export::Ok(__val)
                                                                    => __val,
                                                                    _serde::export::Err(__err)
                                                                    => {
                                                                        return _serde::export::Err(__err);
                                                                    }
                                                                };
                                                        }
                                                    }
                                                }
                                                let __field0 =
                                                    match __field0 {
                                                        _serde::export::Some(__field0)
                                                        => __field0,
                                                        _serde::export::None
                                                        =>
                                                        match _serde::private::de::missing_field("event_type")
                                                            {
                                                            _serde::export::Ok(__val)
                                                            => __val,
                                                            _serde::export::Err(__err)
                                                            => {
                                                                return _serde::export::Err(__err);
                                                            }
                                                        },
                                                    };
                                                let __field1 =
                                                    match __field1 {
                                                        _serde::export::Some(__field1)
                                                        => __field1,
                                                        _serde::export::None
                                                        =>
                                                        match _serde::private::de::missing_field("event_namespace")
                                                            {
                                                            _serde::export::Ok(__val)
                                                            => __val,
                                                            _serde::export::Err(__err)
                                                            => {
                                                                return _serde::export::Err(__err);
                                                            }
                                                        },
                                                    };
                                                _serde::export::Ok(EventIdent{event_type:
                                                                                  __field0,
                                                                              event_namespace:
                                                                                  __field1,})
                                            }
                                        }
                                        const FIELDS: &'static [&'static str]
                                              =
                                            &["event_type",
                                              "event_namespace"];
                                        _serde::Deserializer::deserialize_struct(__deserializer,
                                                                                 "EventIdent",
                                                                                 FIELDS,
                                                                                 __Visitor{marker:
                                                                                               _serde::export::PhantomData::<EventIdent>,
                                                                                           lifetime:
                                                                                               _serde::export::PhantomData,})
                                    }
                                }
                            };
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::std::clone::Clone for EventIdent {
                            #[inline]
                            fn clone(&self) -> EventIdent {
                                match *self {
                                    EventIdent {
                                    event_type: ref __self_0_0,
                                    event_namespace: ref __self_0_1 } =>
                                    EventIdent{event_type:
                                                   ::std::clone::Clone::clone(&(*__self_0_0)),
                                               event_namespace:
                                                   ::std::clone::Clone::clone(&(*__self_0_1)),},
                                }
                            }
                        }
                        struct Helper<'a> {
                            #[serde(rename = "type")]
                            _event_namespace_and_type: Option<String>,
                            #[serde(flatten)]
                            _event_ident: Option<EventIdent>,
                            field: &'a str,
                        }
                        #[allow(non_upper_case_globals,
                                unused_attributes,
                                unused_qualifications)]
                        const _IMPL_DESERIALIZE_FOR_Helper: () =
                            {
                                #[allow(unknown_lints)]
                                #[allow(rust_2018_idioms)]
                                extern crate serde as _serde;
                                #[allow(unused_macros)]
                                macro_rules! try(( $ __expr : expr ) => {
                                                 match $ __expr {
                                                 _serde :: export :: Ok (
                                                 __val ) => __val , _serde ::
                                                 export :: Err ( __err ) => {
                                                 return _serde :: export ::
                                                 Err ( __err ) ; } } });
                                #[automatically_derived]
                                impl <'de: 'a, 'a> _serde::Deserialize<'de>
                                 for Helper<'a> {
                                    fn deserialize<__D>(__deserializer: __D)
                                     ->
                                         _serde::export::Result<Self,
                                                                __D::Error>
                                     where __D: _serde::Deserializer<'de> {
                                        #[allow(non_camel_case_types)]
                                        enum __Field<'de> {
                                            __field0,
                                            __field2,
                                            __other(_serde::private::de::Content<'de>),
                                        }
                                        struct __FieldVisitor;
                                        impl <'de> _serde::de::Visitor<'de>
                                         for __FieldVisitor {
                                            type
                                            Value
                                            =
                                            __Field<'de>;
                                            fn expecting(&self,
                                                         __formatter:
                                                             &mut _serde::export::Formatter)
                                             -> _serde::export::fmt::Result {
                                                _serde::export::Formatter::write_str(__formatter,
                                                                                     "field identifier")
                                            }
                                            fn visit_bool<__E>(self,
                                                               __value: bool)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::Bool(__value)))
                                            }
                                            fn visit_i8<__E>(self,
                                                             __value: i8)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::I8(__value)))
                                            }
                                            fn visit_i16<__E>(self,
                                                              __value: i16)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::I16(__value)))
                                            }
                                            fn visit_i32<__E>(self,
                                                              __value: i32)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::I32(__value)))
                                            }
                                            fn visit_i64<__E>(self,
                                                              __value: i64)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::I64(__value)))
                                            }
                                            fn visit_u8<__E>(self,
                                                             __value: u8)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::U8(__value)))
                                            }
                                            fn visit_u16<__E>(self,
                                                              __value: u16)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::U16(__value)))
                                            }
                                            fn visit_u32<__E>(self,
                                                              __value: u32)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::U32(__value)))
                                            }
                                            fn visit_u64<__E>(self,
                                                              __value: u64)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::U64(__value)))
                                            }
                                            fn visit_f32<__E>(self,
                                                              __value: f32)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::F32(__value)))
                                            }
                                            fn visit_f64<__E>(self,
                                                              __value: f64)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::F64(__value)))
                                            }
                                            fn visit_char<__E>(self,
                                                               __value: char)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::Char(__value)))
                                            }
                                            fn visit_unit<__E>(self)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::Unit))
                                            }
                                            fn visit_borrowed_str<__E>(self,
                                                                       __value:
                                                                           &'de str)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    "type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    "field" =>
                                                    _serde::export::Ok(__Field::__field2),
                                                    _ => {
                                                        let __value =
                                                            _serde::private::de::Content::Str(__value);
                                                        _serde::export::Ok(__Field::__other(__value))
                                                    }
                                                }
                                            }
                                            fn visit_borrowed_bytes<__E>(self,
                                                                         __value:
                                                                             &'de [u8])
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    b"type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    b"field" =>
                                                    _serde::export::Ok(__Field::__field2),
                                                    _ => {
                                                        let __value =
                                                            _serde::private::de::Content::Bytes(__value);
                                                        _serde::export::Ok(__Field::__other(__value))
                                                    }
                                                }
                                            }
                                            fn visit_str<__E>(self,
                                                              __value: &str)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    "type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    "field" =>
                                                    _serde::export::Ok(__Field::__field2),
                                                    _ => {
                                                        let __value =
                                                            _serde::private::de::Content::String(__value.to_string());
                                                        _serde::export::Ok(__Field::__other(__value))
                                                    }
                                                }
                                            }
                                            fn visit_bytes<__E>(self,
                                                                __value:
                                                                    &[u8])
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    b"type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    b"field" =>
                                                    _serde::export::Ok(__Field::__field2),
                                                    _ => {
                                                        let __value =
                                                            _serde::private::de::Content::ByteBuf(__value.to_vec());
                                                        _serde::export::Ok(__Field::__other(__value))
                                                    }
                                                }
                                            }
                                        }
                                        impl <'de> _serde::Deserialize<'de>
                                         for __Field<'de> {
                                            #[inline]
                                            fn deserialize<__D>(__deserializer:
                                                                    __D)
                                             ->
                                                 _serde::export::Result<Self,
                                                                        __D::Error>
                                             where
                                             __D: _serde::Deserializer<'de> {
                                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                                                                             __FieldVisitor)
                                            }
                                        }
                                        struct __Visitor<'de: 'a, 'a> {
                                            marker: _serde::export::PhantomData<Helper<'a>>,
                                            lifetime: _serde::export::PhantomData<&'de ()>,
                                        }
                                        impl <'de: 'a, 'a>
                                         _serde::de::Visitor<'de> for
                                         __Visitor<'de, 'a> {
                                            type
                                            Value
                                            =
                                            Helper<'a>;
                                            fn expecting(&self,
                                                         __formatter:
                                                             &mut _serde::export::Formatter)
                                             -> _serde::export::fmt::Result {
                                                _serde::export::Formatter::write_str(__formatter,
                                                                                     "struct Helper")
                                            }
                                            #[inline]
                                            fn visit_map<__A>(self,
                                                              mut __map: __A)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __A::Error>
                                             where
                                             __A: _serde::de::MapAccess<'de> {
                                                let mut __field0:
                                                        _serde::export::Option<Option<String>> =
                                                    _serde::export::None;
                                                let mut __field2:
                                                        _serde::export::Option<&'a str> =
                                                    _serde::export::None;
                                                let mut __collect =
                                                    _serde::export::Vec::<_serde::export::Option<(_serde::private::de::Content,
                                                                                                  _serde::private::de::Content)>>::new();
                                                while let _serde::export::Some(__key)
                                                          =
                                                          match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                                              {
                                                              _serde::export::Ok(__val)
                                                              => __val,
                                                              _serde::export::Err(__err)
                                                              => {
                                                                  return _serde::export::Err(__err);
                                                              }
                                                          } {
                                                    match __key {
                                                        __Field::__field0 => {
                                                            if _serde::export::Option::is_some(&__field0)
                                                               {
                                                                return _serde::export::Err(<__A::Error
                                                                                               as
                                                                                               _serde::de::Error>::duplicate_field("type"));
                                                            }
                                                            __field0 =
                                                                _serde::export::Some(match _serde::de::MapAccess::next_value::<Option<String>>(&mut __map)
                                                                                         {
                                                                                         _serde::export::Ok(__val)
                                                                                         =>
                                                                                         __val,
                                                                                         _serde::export::Err(__err)
                                                                                         =>
                                                                                         {
                                                                                             return _serde::export::Err(__err);
                                                                                         }
                                                                                     });
                                                        }
                                                        __Field::__field2 => {
                                                            if _serde::export::Option::is_some(&__field2)
                                                               {
                                                                return _serde::export::Err(<__A::Error
                                                                                               as
                                                                                               _serde::de::Error>::duplicate_field("field"));
                                                            }
                                                            __field2 =
                                                                _serde::export::Some(match _serde::de::MapAccess::next_value::<&'a str>(&mut __map)
                                                                                         {
                                                                                         _serde::export::Ok(__val)
                                                                                         =>
                                                                                         __val,
                                                                                         _serde::export::Err(__err)
                                                                                         =>
                                                                                         {
                                                                                             return _serde::export::Err(__err);
                                                                                         }
                                                                                     });
                                                        }
                                                        __Field::__other(__name)
                                                        => {
                                                            __collect.push(_serde::export::Some((__name,
                                                                                                 match _serde::de::MapAccess::next_value(&mut __map)
                                                                                                     {
                                                                                                     _serde::export::Ok(__val)
                                                                                                     =>
                                                                                                     __val,
                                                                                                     _serde::export::Err(__err)
                                                                                                     =>
                                                                                                     {
                                                                                                         return _serde::export::Err(__err);
                                                                                                     }
                                                                                                 })));
                                                        }
                                                    }
                                                }
                                                let __field0 =
                                                    match __field0 {
                                                        _serde::export::Some(__field0)
                                                        => __field0,
                                                        _serde::export::None
                                                        =>
                                                        match _serde::private::de::missing_field("type")
                                                            {
                                                            _serde::export::Ok(__val)
                                                            => __val,
                                                            _serde::export::Err(__err)
                                                            => {
                                                                return _serde::export::Err(__err);
                                                            }
                                                        },
                                                    };
                                                let __field2 =
                                                    match __field2 {
                                                        _serde::export::Some(__field2)
                                                        => __field2,
                                                        _serde::export::None
                                                        =>
                                                        match _serde::private::de::missing_field("field")
                                                            {
                                                            _serde::export::Ok(__val)
                                                            => __val,
                                                            _serde::export::Err(__err)
                                                            => {
                                                                return _serde::export::Err(__err);
                                                            }
                                                        },
                                                    };
                                                let __field1:
                                                        Option<EventIdent> =
                                                    match _serde::de::Deserialize::deserialize(_serde::private::de::FlatMapDeserializer(&mut __collect,
                                                                                                                                        _serde::export::PhantomData))
                                                        {
                                                        _serde::export::Ok(__val)
                                                        => __val,
                                                        _serde::export::Err(__err)
                                                        => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    };
                                                _serde::export::Ok(Helper{_event_namespace_and_type:
                                                                              __field0,
                                                                          _event_ident:
                                                                              __field1,
                                                                          field:
                                                                              __field2,})
                                            }
                                        }
                                        _serde::Deserializer::deserialize_map(__deserializer,
                                                                              __Visitor{marker:
                                                                                            _serde::export::PhantomData::<Helper<'a>>,
                                                                                        lifetime:
                                                                                            _serde::export::PhantomData,})
                                    }
                                }
                            };
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl <'a> ::std::clone::Clone for Helper<'a> {
                            #[inline]
                            fn clone(&self) -> Helper<'a> {
                                match *self {
                                    Helper {
                                    _event_namespace_and_type: ref __self_0_0,
                                    _event_ident: ref __self_0_1,
                                    field: ref __self_0_2 } =>
                                    Helper{_event_namespace_and_type:
                                               ::std::clone::Clone::clone(&(*__self_0_0)),
                                           _event_ident:
                                               ::std::clone::Clone::clone(&(*__self_0_1)),
                                           field:
                                               ::std::clone::Clone::clone(&(*__self_0_2)),},
                                }
                            }
                        }
                        let helper =
                            Helper::deserialize(deserializer).map_err(de::Error::custom)?;
                        let ident =
                            if let Some(ident) = helper._event_ident {
                                ident
                            } else if let Some(ns_and_ty) =
                             helper._event_namespace_and_type {
                                let parts =
                                    ns_and_ty.split('.').map(|part|
                                                                 String::from(part)).collect::<Vec<String>>();
                                EventIdent{event_namespace: parts[0].clone(),
                                           event_type: parts[1].clone(),}
                            } else {
                                return Err(de::Error::custom("No event identifier found"));
                            };
                        if ident.event_type == "LifetimeStruct" &&
                               ident.event_namespace == "test_lifetimes" {
                            Ok(LifetimeStruct{field: helper.field,})
                        } else {
                            Err(de::Error::custom("Incorrect event identifier"))
                        }
                    }
                }
            };
        #[automatically_derived]
        #[allow(unused_qualifications)]
        #[allow(unused)]
        impl <'a> ::std::fmt::Debug for LifetimeStruct<'a> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
             -> ::std::fmt::Result {
                match *self {
                    LifetimeStruct { field: ref __self_0_0 } => {
                        let mut debug_trait_builder =
                            f.debug_struct("LifetimeStruct");
                        let _ =
                            debug_trait_builder.field("field",
                                                      &&(*__self_0_0));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        /// Set of all events in the domain
        pub enum TestEvents { Inc(Event<TestEvent>), }
        #[allow(non_upper_case_globals, unused_attributes, unused_imports)]
        const _IMPL_EVENT_STORE_ENUM_FOR_TestEvents: () =
            {
                extern crate serde;
                extern crate event_store_derive_internals;
                use serde::ser;
                use serde::de::{Deserialize, Deserializer, IntoDeserializer};
                use serde::ser::{Serialize, Serializer, SerializeMap};
                impl event_store_derive_internals::Events for TestEvents { }
                impl <'de> Serialize for TestEvents {
                    fn serialize<S>(&self, serializer: S)
                     -> Result<S::Ok, S::Error> where S: Serializer {
                        match self {
                            TestEvents::Inc(evt) =>
                            evt.serialize(serializer).map_err(ser::Error::custom),
                        }
                    }
                }
                impl <'de> serde::Deserialize<'de> for TestEvents {
                    fn deserialize<__D>(deserializer: __D)
                     -> Result<Self, __D::Error> where
                     __D: Deserializer<'de> {
                        use serde::de;
                        #[serde(untagged)]
                        enum Output { Inc(Event<TestEvent>), }
                        #[allow(non_upper_case_globals,
                                unused_attributes,
                                unused_qualifications)]
                        const _IMPL_DESERIALIZE_FOR_Output: () =
                            {
                                #[allow(unknown_lints)]
                                #[allow(rust_2018_idioms)]
                                extern crate serde as _serde;
                                #[allow(unused_macros)]
                                macro_rules! try(( $ __expr : expr ) => {
                                                 match $ __expr {
                                                 _serde :: export :: Ok (
                                                 __val ) => __val , _serde ::
                                                 export :: Err ( __err ) => {
                                                 return _serde :: export ::
                                                 Err ( __err ) ; } } });
                                #[automatically_derived]
                                impl <'de> _serde::Deserialize<'de> for Output
                                 {
                                    fn deserialize<__D>(__deserializer: __D)
                                     ->
                                         _serde::export::Result<Self,
                                                                __D::Error>
                                     where __D: _serde::Deserializer<'de> {
                                        let __content =
                                            match <_serde::private::de::Content
                                                      as
                                                      _serde::Deserialize>::deserialize(__deserializer)
                                                {
                                                _serde::export::Ok(__val) =>
                                                __val,
                                                _serde::export::Err(__err) =>
                                                {
                                                    return _serde::export::Err(__err);
                                                }
                                            };
                                        if let _serde::export::Ok(__ok) =
                                               _serde::export::Result::map(<Event<TestEvent>
                                                                               as
                                                                               _serde::Deserialize>::deserialize(_serde::private::de::ContentRefDeserializer::<__D::Error>::new(&__content)),
                                                                           Output::Inc)
                                               {
                                            return _serde::export::Ok(__ok);
                                        }
                                        _serde::export::Err(_serde::de::Error::custom("data did not match any variant of untagged enum Output"))
                                    }
                                }
                            };
                        let out =
                            Output::deserialize(deserializer).map_err(de::Error::custom)?;
                        let mapped =
                            match out {
                                Output::Inc(evt) => TestEvents::Inc(evt),
                            };
                        Ok(mapped)
                    }
                }
            };
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::fmt::Debug for TestEvents {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
             -> ::std::fmt::Result {
                match (&*self,) {
                    (&TestEvents::Inc(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Inc");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        #[event_store(namespace = "some_namespace")]
        pub struct TestEvent {
            pub num: i32,
        }
        #[allow(non_upper_case_globals, unused_attributes, unused_imports)]
        const _IMPL_EVENT_STORE_STRUCT_FOR_TestEvent: () =
            {
                extern crate serde;
                extern crate serde_derive;
                extern crate event_store_derive_internals;
                use serde::ser;
                use serde::de::{Deserialize, Deserializer};
                use serde::ser::{Serialize, Serializer, SerializeMap};
                impl event_store_derive_internals::EventData for TestEvent {
                    fn event_namespace_and_type() -> &'static str {
                        "some_namespace.TestEvent"
                    }
                    fn event_namespace() -> &'static str { "some_namespace" }
                    fn event_type() -> &'static str { "TestEvent" }
                }
                impl Serialize for TestEvent {
                    fn serialize<S>(&self, serializer: S)
                     -> Result<S::Ok, S::Error> where S: Serializer {
                        let mut map = serializer.serialize_map(Some(4usize))?;
                        for (k, v) in self {
                            map.serialize_entry("#field_idents", self.num)?;
                        }
                        map.serialize_entry("event_namespace_and_type",
                                            "some_namespace.TestEvent")?;
                        map.serialize_entry("event_namespace",
                                            "some_namespace")?;
                        map.serialize_entry("event_type", "TestEvent")?;
                        map.end()
                    }
                }
                impl <'de> serde::Deserialize<'de> for TestEvent {
                    fn deserialize<__D>(deserializer: __D)
                     -> serde::export::Result<Self, __D::Error> where
                     __D: serde::Deserializer<'de> {
                        use serde::de;
                        struct EventIdent {
                            event_type: String,
                            event_namespace: String,
                        }
                        #[allow(non_upper_case_globals,
                                unused_attributes,
                                unused_qualifications)]
                        const _IMPL_DESERIALIZE_FOR_EventIdent: () =
                            {
                                #[allow(unknown_lints)]
                                #[allow(rust_2018_idioms)]
                                extern crate serde as _serde;
                                #[allow(unused_macros)]
                                macro_rules! try(( $ __expr : expr ) => {
                                                 match $ __expr {
                                                 _serde :: export :: Ok (
                                                 __val ) => __val , _serde ::
                                                 export :: Err ( __err ) => {
                                                 return _serde :: export ::
                                                 Err ( __err ) ; } } });
                                #[automatically_derived]
                                impl <'de> _serde::Deserialize<'de> for
                                 EventIdent {
                                    fn deserialize<__D>(__deserializer: __D)
                                     ->
                                         _serde::export::Result<Self,
                                                                __D::Error>
                                     where __D: _serde::Deserializer<'de> {
                                        #[allow(non_camel_case_types)]
                                        enum __Field {
                                            __field0,
                                            __field1,
                                            __ignore,
                                        }
                                        struct __FieldVisitor;
                                        impl <'de> _serde::de::Visitor<'de>
                                         for __FieldVisitor {
                                            type
                                            Value
                                            =
                                            __Field;
                                            fn expecting(&self,
                                                         __formatter:
                                                             &mut _serde::export::Formatter)
                                             -> _serde::export::fmt::Result {
                                                _serde::export::Formatter::write_str(__formatter,
                                                                                     "field identifier")
                                            }
                                            fn visit_u64<__E>(self,
                                                              __value: u64)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    0u64 =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    1u64 =>
                                                    _serde::export::Ok(__Field::__field1),
                                                    _ =>
                                                    _serde::export::Err(_serde::de::Error::invalid_value(_serde::de::Unexpected::Unsigned(__value),
                                                                                                         &"field index 0 <= i < 2")),
                                                }
                                            }
                                            fn visit_str<__E>(self,
                                                              __value: &str)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    "event_type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    "event_namespace" =>
                                                    _serde::export::Ok(__Field::__field1),
                                                    _ => {
                                                        _serde::export::Ok(__Field::__ignore)
                                                    }
                                                }
                                            }
                                            fn visit_bytes<__E>(self,
                                                                __value:
                                                                    &[u8])
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    b"event_type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    b"event_namespace" =>
                                                    _serde::export::Ok(__Field::__field1),
                                                    _ => {
                                                        _serde::export::Ok(__Field::__ignore)
                                                    }
                                                }
                                            }
                                        }
                                        impl <'de> _serde::Deserialize<'de>
                                         for __Field {
                                            #[inline]
                                            fn deserialize<__D>(__deserializer:
                                                                    __D)
                                             ->
                                                 _serde::export::Result<Self,
                                                                        __D::Error>
                                             where
                                             __D: _serde::Deserializer<'de> {
                                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                                                                             __FieldVisitor)
                                            }
                                        }
                                        struct __Visitor<'de> {
                                            marker: _serde::export::PhantomData<EventIdent>,
                                            lifetime: _serde::export::PhantomData<&'de ()>,
                                        }
                                        impl <'de> _serde::de::Visitor<'de>
                                         for __Visitor<'de> {
                                            type
                                            Value
                                            =
                                            EventIdent;
                                            fn expecting(&self,
                                                         __formatter:
                                                             &mut _serde::export::Formatter)
                                             -> _serde::export::fmt::Result {
                                                _serde::export::Formatter::write_str(__formatter,
                                                                                     "struct EventIdent")
                                            }
                                            #[inline]
                                            fn visit_seq<__A>(self,
                                                              mut __seq: __A)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __A::Error>
                                             where
                                             __A: _serde::de::SeqAccess<'de> {
                                                let __field0 =
                                                    match match _serde::de::SeqAccess::next_element::<String>(&mut __seq)
                                                              {
                                                              _serde::export::Ok(__val)
                                                              => __val,
                                                              _serde::export::Err(__err)
                                                              => {
                                                                  return _serde::export::Err(__err);
                                                              }
                                                          } {
                                                        _serde::export::Some(__value)
                                                        => __value,
                                                        _serde::export::None
                                                        => {
                                                            return _serde::export::Err(_serde::de::Error::invalid_length(0usize,
                                                                                                                         &"struct EventIdent with 2 elements"));
                                                        }
                                                    };
                                                let __field1 =
                                                    match match _serde::de::SeqAccess::next_element::<String>(&mut __seq)
                                                              {
                                                              _serde::export::Ok(__val)
                                                              => __val,
                                                              _serde::export::Err(__err)
                                                              => {
                                                                  return _serde::export::Err(__err);
                                                              }
                                                          } {
                                                        _serde::export::Some(__value)
                                                        => __value,
                                                        _serde::export::None
                                                        => {
                                                            return _serde::export::Err(_serde::de::Error::invalid_length(1usize,
                                                                                                                         &"struct EventIdent with 2 elements"));
                                                        }
                                                    };
                                                _serde::export::Ok(EventIdent{event_type:
                                                                                  __field0,
                                                                              event_namespace:
                                                                                  __field1,})
                                            }
                                            #[inline]
                                            fn visit_map<__A>(self,
                                                              mut __map: __A)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __A::Error>
                                             where
                                             __A: _serde::de::MapAccess<'de> {
                                                let mut __field0:
                                                        _serde::export::Option<String> =
                                                    _serde::export::None;
                                                let mut __field1:
                                                        _serde::export::Option<String> =
                                                    _serde::export::None;
                                                while let _serde::export::Some(__key)
                                                          =
                                                          match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                                              {
                                                              _serde::export::Ok(__val)
                                                              => __val,
                                                              _serde::export::Err(__err)
                                                              => {
                                                                  return _serde::export::Err(__err);
                                                              }
                                                          } {
                                                    match __key {
                                                        __Field::__field0 => {
                                                            if _serde::export::Option::is_some(&__field0)
                                                               {
                                                                return _serde::export::Err(<__A::Error
                                                                                               as
                                                                                               _serde::de::Error>::duplicate_field("event_type"));
                                                            }
                                                            __field0 =
                                                                _serde::export::Some(match _serde::de::MapAccess::next_value::<String>(&mut __map)
                                                                                         {
                                                                                         _serde::export::Ok(__val)
                                                                                         =>
                                                                                         __val,
                                                                                         _serde::export::Err(__err)
                                                                                         =>
                                                                                         {
                                                                                             return _serde::export::Err(__err);
                                                                                         }
                                                                                     });
                                                        }
                                                        __Field::__field1 => {
                                                            if _serde::export::Option::is_some(&__field1)
                                                               {
                                                                return _serde::export::Err(<__A::Error
                                                                                               as
                                                                                               _serde::de::Error>::duplicate_field("event_namespace"));
                                                            }
                                                            __field1 =
                                                                _serde::export::Some(match _serde::de::MapAccess::next_value::<String>(&mut __map)
                                                                                         {
                                                                                         _serde::export::Ok(__val)
                                                                                         =>
                                                                                         __val,
                                                                                         _serde::export::Err(__err)
                                                                                         =>
                                                                                         {
                                                                                             return _serde::export::Err(__err);
                                                                                         }
                                                                                     });
                                                        }
                                                        _ => {
                                                            let _ =
                                                                match _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)
                                                                    {
                                                                    _serde::export::Ok(__val)
                                                                    => __val,
                                                                    _serde::export::Err(__err)
                                                                    => {
                                                                        return _serde::export::Err(__err);
                                                                    }
                                                                };
                                                        }
                                                    }
                                                }
                                                let __field0 =
                                                    match __field0 {
                                                        _serde::export::Some(__field0)
                                                        => __field0,
                                                        _serde::export::None
                                                        =>
                                                        match _serde::private::de::missing_field("event_type")
                                                            {
                                                            _serde::export::Ok(__val)
                                                            => __val,
                                                            _serde::export::Err(__err)
                                                            => {
                                                                return _serde::export::Err(__err);
                                                            }
                                                        },
                                                    };
                                                let __field1 =
                                                    match __field1 {
                                                        _serde::export::Some(__field1)
                                                        => __field1,
                                                        _serde::export::None
                                                        =>
                                                        match _serde::private::de::missing_field("event_namespace")
                                                            {
                                                            _serde::export::Ok(__val)
                                                            => __val,
                                                            _serde::export::Err(__err)
                                                            => {
                                                                return _serde::export::Err(__err);
                                                            }
                                                        },
                                                    };
                                                _serde::export::Ok(EventIdent{event_type:
                                                                                  __field0,
                                                                              event_namespace:
                                                                                  __field1,})
                                            }
                                        }
                                        const FIELDS: &'static [&'static str]
                                              =
                                            &["event_type",
                                              "event_namespace"];
                                        _serde::Deserializer::deserialize_struct(__deserializer,
                                                                                 "EventIdent",
                                                                                 FIELDS,
                                                                                 __Visitor{marker:
                                                                                               _serde::export::PhantomData::<EventIdent>,
                                                                                           lifetime:
                                                                                               _serde::export::PhantomData,})
                                    }
                                }
                            };
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::std::clone::Clone for EventIdent {
                            #[inline]
                            fn clone(&self) -> EventIdent {
                                match *self {
                                    EventIdent {
                                    event_type: ref __self_0_0,
                                    event_namespace: ref __self_0_1 } =>
                                    EventIdent{event_type:
                                                   ::std::clone::Clone::clone(&(*__self_0_0)),
                                               event_namespace:
                                                   ::std::clone::Clone::clone(&(*__self_0_1)),},
                                }
                            }
                        }
                        struct Helper {
                            #[serde(rename = "type")]
                            _event_namespace_and_type: Option<String>,
                            #[serde(flatten)]
                            _event_ident: Option<EventIdent>,
                            pub num: i32,
                        }
                        #[allow(non_upper_case_globals,
                                unused_attributes,
                                unused_qualifications)]
                        const _IMPL_DESERIALIZE_FOR_Helper: () =
                            {
                                #[allow(unknown_lints)]
                                #[allow(rust_2018_idioms)]
                                extern crate serde as _serde;
                                #[allow(unused_macros)]
                                macro_rules! try(( $ __expr : expr ) => {
                                                 match $ __expr {
                                                 _serde :: export :: Ok (
                                                 __val ) => __val , _serde ::
                                                 export :: Err ( __err ) => {
                                                 return _serde :: export ::
                                                 Err ( __err ) ; } } });
                                #[automatically_derived]
                                impl <'de> _serde::Deserialize<'de> for Helper
                                 {
                                    fn deserialize<__D>(__deserializer: __D)
                                     ->
                                         _serde::export::Result<Self,
                                                                __D::Error>
                                     where __D: _serde::Deserializer<'de> {
                                        #[allow(non_camel_case_types)]
                                        enum __Field<'de> {
                                            __field0,
                                            __field2,
                                            __other(_serde::private::de::Content<'de>),
                                        }
                                        struct __FieldVisitor;
                                        impl <'de> _serde::de::Visitor<'de>
                                         for __FieldVisitor {
                                            type
                                            Value
                                            =
                                            __Field<'de>;
                                            fn expecting(&self,
                                                         __formatter:
                                                             &mut _serde::export::Formatter)
                                             -> _serde::export::fmt::Result {
                                                _serde::export::Formatter::write_str(__formatter,
                                                                                     "field identifier")
                                            }
                                            fn visit_bool<__E>(self,
                                                               __value: bool)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::Bool(__value)))
                                            }
                                            fn visit_i8<__E>(self,
                                                             __value: i8)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::I8(__value)))
                                            }
                                            fn visit_i16<__E>(self,
                                                              __value: i16)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::I16(__value)))
                                            }
                                            fn visit_i32<__E>(self,
                                                              __value: i32)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::I32(__value)))
                                            }
                                            fn visit_i64<__E>(self,
                                                              __value: i64)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::I64(__value)))
                                            }
                                            fn visit_u8<__E>(self,
                                                             __value: u8)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::U8(__value)))
                                            }
                                            fn visit_u16<__E>(self,
                                                              __value: u16)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::U16(__value)))
                                            }
                                            fn visit_u32<__E>(self,
                                                              __value: u32)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::U32(__value)))
                                            }
                                            fn visit_u64<__E>(self,
                                                              __value: u64)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::U64(__value)))
                                            }
                                            fn visit_f32<__E>(self,
                                                              __value: f32)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::F32(__value)))
                                            }
                                            fn visit_f64<__E>(self,
                                                              __value: f64)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::F64(__value)))
                                            }
                                            fn visit_char<__E>(self,
                                                               __value: char)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::Char(__value)))
                                            }
                                            fn visit_unit<__E>(self)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                _serde::export::Ok(__Field::__other(_serde::private::de::Content::Unit))
                                            }
                                            fn visit_borrowed_str<__E>(self,
                                                                       __value:
                                                                           &'de str)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    "type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    "num" =>
                                                    _serde::export::Ok(__Field::__field2),
                                                    _ => {
                                                        let __value =
                                                            _serde::private::de::Content::Str(__value);
                                                        _serde::export::Ok(__Field::__other(__value))
                                                    }
                                                }
                                            }
                                            fn visit_borrowed_bytes<__E>(self,
                                                                         __value:
                                                                             &'de [u8])
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    b"type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    b"num" =>
                                                    _serde::export::Ok(__Field::__field2),
                                                    _ => {
                                                        let __value =
                                                            _serde::private::de::Content::Bytes(__value);
                                                        _serde::export::Ok(__Field::__other(__value))
                                                    }
                                                }
                                            }
                                            fn visit_str<__E>(self,
                                                              __value: &str)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    "type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    "num" =>
                                                    _serde::export::Ok(__Field::__field2),
                                                    _ => {
                                                        let __value =
                                                            _serde::private::de::Content::String(__value.to_string());
                                                        _serde::export::Ok(__Field::__other(__value))
                                                    }
                                                }
                                            }
                                            fn visit_bytes<__E>(self,
                                                                __value:
                                                                    &[u8])
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __E>
                                             where __E: _serde::de::Error {
                                                match __value {
                                                    b"type" =>
                                                    _serde::export::Ok(__Field::__field0),
                                                    b"num" =>
                                                    _serde::export::Ok(__Field::__field2),
                                                    _ => {
                                                        let __value =
                                                            _serde::private::de::Content::ByteBuf(__value.to_vec());
                                                        _serde::export::Ok(__Field::__other(__value))
                                                    }
                                                }
                                            }
                                        }
                                        impl <'de> _serde::Deserialize<'de>
                                         for __Field<'de> {
                                            #[inline]
                                            fn deserialize<__D>(__deserializer:
                                                                    __D)
                                             ->
                                                 _serde::export::Result<Self,
                                                                        __D::Error>
                                             where
                                             __D: _serde::Deserializer<'de> {
                                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                                                                             __FieldVisitor)
                                            }
                                        }
                                        struct __Visitor<'de> {
                                            marker: _serde::export::PhantomData<Helper>,
                                            lifetime: _serde::export::PhantomData<&'de ()>,
                                        }
                                        impl <'de> _serde::de::Visitor<'de>
                                         for __Visitor<'de> {
                                            type
                                            Value
                                            =
                                            Helper;
                                            fn expecting(&self,
                                                         __formatter:
                                                             &mut _serde::export::Formatter)
                                             -> _serde::export::fmt::Result {
                                                _serde::export::Formatter::write_str(__formatter,
                                                                                     "struct Helper")
                                            }
                                            #[inline]
                                            fn visit_map<__A>(self,
                                                              mut __map: __A)
                                             ->
                                                 _serde::export::Result<Self::Value,
                                                                        __A::Error>
                                             where
                                             __A: _serde::de::MapAccess<'de> {
                                                let mut __field0:
                                                        _serde::export::Option<Option<String>> =
                                                    _serde::export::None;
                                                let mut __field2:
                                                        _serde::export::Option<i32> =
                                                    _serde::export::None;
                                                let mut __collect =
                                                    _serde::export::Vec::<_serde::export::Option<(_serde::private::de::Content,
                                                                                                  _serde::private::de::Content)>>::new();
                                                while let _serde::export::Some(__key)
                                                          =
                                                          match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                                              {
                                                              _serde::export::Ok(__val)
                                                              => __val,
                                                              _serde::export::Err(__err)
                                                              => {
                                                                  return _serde::export::Err(__err);
                                                              }
                                                          } {
                                                    match __key {
                                                        __Field::__field0 => {
                                                            if _serde::export::Option::is_some(&__field0)
                                                               {
                                                                return _serde::export::Err(<__A::Error
                                                                                               as
                                                                                               _serde::de::Error>::duplicate_field("type"));
                                                            }
                                                            __field0 =
                                                                _serde::export::Some(match _serde::de::MapAccess::next_value::<Option<String>>(&mut __map)
                                                                                         {
                                                                                         _serde::export::Ok(__val)
                                                                                         =>
                                                                                         __val,
                                                                                         _serde::export::Err(__err)
                                                                                         =>
                                                                                         {
                                                                                             return _serde::export::Err(__err);
                                                                                         }
                                                                                     });
                                                        }
                                                        __Field::__field2 => {
                                                            if _serde::export::Option::is_some(&__field2)
                                                               {
                                                                return _serde::export::Err(<__A::Error
                                                                                               as
                                                                                               _serde::de::Error>::duplicate_field("num"));
                                                            }
                                                            __field2 =
                                                                _serde::export::Some(match _serde::de::MapAccess::next_value::<i32>(&mut __map)
                                                                                         {
                                                                                         _serde::export::Ok(__val)
                                                                                         =>
                                                                                         __val,
                                                                                         _serde::export::Err(__err)
                                                                                         =>
                                                                                         {
                                                                                             return _serde::export::Err(__err);
                                                                                         }
                                                                                     });
                                                        }
                                                        __Field::__other(__name)
                                                        => {
                                                            __collect.push(_serde::export::Some((__name,
                                                                                                 match _serde::de::MapAccess::next_value(&mut __map)
                                                                                                     {
                                                                                                     _serde::export::Ok(__val)
                                                                                                     =>
                                                                                                     __val,
                                                                                                     _serde::export::Err(__err)
                                                                                                     =>
                                                                                                     {
                                                                                                         return _serde::export::Err(__err);
                                                                                                     }
                                                                                                 })));
                                                        }
                                                    }
                                                }
                                                let __field0 =
                                                    match __field0 {
                                                        _serde::export::Some(__field0)
                                                        => __field0,
                                                        _serde::export::None
                                                        =>
                                                        match _serde::private::de::missing_field("type")
                                                            {
                                                            _serde::export::Ok(__val)
                                                            => __val,
                                                            _serde::export::Err(__err)
                                                            => {
                                                                return _serde::export::Err(__err);
                                                            }
                                                        },
                                                    };
                                                let __field2 =
                                                    match __field2 {
                                                        _serde::export::Some(__field2)
                                                        => __field2,
                                                        _serde::export::None
                                                        =>
                                                        match _serde::private::de::missing_field("num")
                                                            {
                                                            _serde::export::Ok(__val)
                                                            => __val,
                                                            _serde::export::Err(__err)
                                                            => {
                                                                return _serde::export::Err(__err);
                                                            }
                                                        },
                                                    };
                                                let __field1:
                                                        Option<EventIdent> =
                                                    match _serde::de::Deserialize::deserialize(_serde::private::de::FlatMapDeserializer(&mut __collect,
                                                                                                                                        _serde::export::PhantomData))
                                                        {
                                                        _serde::export::Ok(__val)
                                                        => __val,
                                                        _serde::export::Err(__err)
                                                        => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    };
                                                _serde::export::Ok(Helper{_event_namespace_and_type:
                                                                              __field0,
                                                                          _event_ident:
                                                                              __field1,
                                                                          num:
                                                                              __field2,})
                                            }
                                        }
                                        _serde::Deserializer::deserialize_map(__deserializer,
                                                                              __Visitor{marker:
                                                                                            _serde::export::PhantomData::<Helper>,
                                                                                        lifetime:
                                                                                            _serde::export::PhantomData,})
                                    }
                                }
                            };
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::std::clone::Clone for Helper {
                            #[inline]
                            fn clone(&self) -> Helper {
                                match *self {
                                    Helper {
                                    _event_namespace_and_type: ref __self_0_0,
                                    _event_ident: ref __self_0_1,
                                    num: ref __self_0_2 } =>
                                    Helper{_event_namespace_and_type:
                                               ::std::clone::Clone::clone(&(*__self_0_0)),
                                           _event_ident:
                                               ::std::clone::Clone::clone(&(*__self_0_1)),
                                           num:
                                               ::std::clone::Clone::clone(&(*__self_0_2)),},
                                }
                            }
                        }
                        let helper =
                            Helper::deserialize(deserializer).map_err(de::Error::custom)?;
                        let ident =
                            if let Some(ident) = helper._event_ident {
                                ident
                            } else if let Some(ns_and_ty) =
                             helper._event_namespace_and_type {
                                let parts =
                                    ns_and_ty.split('.').map(|part|
                                                                 String::from(part)).collect::<Vec<String>>();
                                EventIdent{event_namespace: parts[0].clone(),
                                           event_type: parts[1].clone(),}
                            } else {
                                return Err(de::Error::custom("No event identifier found"));
                            };
                        if ident.event_type == "TestEvent" &&
                               ident.event_namespace == "some_namespace" {
                            Ok(TestEvent{num: helper.num,})
                        } else {
                            Err(de::Error::custom("Incorrect event identifier"))
                        }
                    }
                }
            };
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::fmt::Debug for TestEvent {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
             -> ::std::fmt::Result {
                match *self {
                    TestEvent { num: ref __self_0_0 } => {
                        let mut debug_trait_builder =
                            f.debug_struct("TestEvent");
                        let _ =
                            debug_trait_builder.field("num", &&(*__self_0_0));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        /// Testing entity for a pretend domain
        #[rustc_copy_clone_marker]
        pub struct TestCounterEntity {
            /// Current counter value
            pub counter: i32,
        }
        #[allow(non_upper_case_globals,
                unused_attributes,
                unused_qualifications)]
        const _IMPL_SERIALIZE_FOR_TestCounterEntity: () =
            {
                #[allow(unknown_lints)]
                #[allow(rust_2018_idioms)]
                extern crate serde as _serde;
                #[allow(unused_macros)]
                macro_rules! try(( $ __expr : expr ) => {
                                 match $ __expr {
                                 _serde :: export :: Ok ( __val ) => __val ,
                                 _serde :: export :: Err ( __err ) => {
                                 return _serde :: export :: Err ( __err ) ; }
                                 } });
                #[automatically_derived]
                impl _serde::Serialize for TestCounterEntity {
                    fn serialize<__S>(&self, __serializer: __S)
                     -> _serde::export::Result<__S::Ok, __S::Error> where
                     __S: _serde::Serializer {
                        let mut __serde_state =
                            match _serde::Serializer::serialize_struct(__serializer,
                                                                       "TestCounterEntity",
                                                                       false
                                                                           as
                                                                           usize
                                                                           +
                                                                           1)
                                {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            };
                        match _serde::ser::SerializeStruct::serialize_field(&mut __serde_state,
                                                                            "counter",
                                                                            &self.counter)
                            {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        };
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        #[allow(non_upper_case_globals,
                unused_attributes,
                unused_qualifications)]
        const _IMPL_DESERIALIZE_FOR_TestCounterEntity: () =
            {
                #[allow(unknown_lints)]
                #[allow(rust_2018_idioms)]
                extern crate serde as _serde;
                #[allow(unused_macros)]
                macro_rules! try(( $ __expr : expr ) => {
                                 match $ __expr {
                                 _serde :: export :: Ok ( __val ) => __val ,
                                 _serde :: export :: Err ( __err ) => {
                                 return _serde :: export :: Err ( __err ) ; }
                                 } });
                #[automatically_derived]
                impl <'de> _serde::Deserialize<'de> for TestCounterEntity {
                    fn deserialize<__D>(__deserializer: __D)
                     -> _serde::export::Result<Self, __D::Error> where
                     __D: _serde::Deserializer<'de> {
                        #[allow(non_camel_case_types)]
                        enum __Field { __field0, __ignore, }
                        struct __FieldVisitor;
                        impl <'de> _serde::de::Visitor<'de> for __FieldVisitor
                         {
                            type
                            Value
                            =
                            __Field;
                            fn expecting(&self,
                                         __formatter:
                                             &mut _serde::export::Formatter)
                             -> _serde::export::fmt::Result {
                                _serde::export::Formatter::write_str(__formatter,
                                                                     "field identifier")
                            }
                            fn visit_u64<__E>(self, __value: u64)
                             -> _serde::export::Result<Self::Value, __E> where
                             __E: _serde::de::Error {
                                match __value {
                                    0u64 =>
                                    _serde::export::Ok(__Field::__field0),
                                    _ =>
                                    _serde::export::Err(_serde::de::Error::invalid_value(_serde::de::Unexpected::Unsigned(__value),
                                                                                         &"field index 0 <= i < 1")),
                                }
                            }
                            fn visit_str<__E>(self, __value: &str)
                             -> _serde::export::Result<Self::Value, __E> where
                             __E: _serde::de::Error {
                                match __value {
                                    "counter" =>
                                    _serde::export::Ok(__Field::__field0),
                                    _ => {
                                        _serde::export::Ok(__Field::__ignore)
                                    }
                                }
                            }
                            fn visit_bytes<__E>(self, __value: &[u8])
                             -> _serde::export::Result<Self::Value, __E> where
                             __E: _serde::de::Error {
                                match __value {
                                    b"counter" =>
                                    _serde::export::Ok(__Field::__field0),
                                    _ => {
                                        _serde::export::Ok(__Field::__ignore)
                                    }
                                }
                            }
                        }
                        impl <'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(__deserializer: __D)
                             -> _serde::export::Result<Self, __D::Error> where
                             __D: _serde::Deserializer<'de> {
                                _serde::Deserializer::deserialize_identifier(__deserializer,
                                                                             __FieldVisitor)
                            }
                        }
                        struct __Visitor<'de> {
                            marker: _serde::export::PhantomData<TestCounterEntity>,
                            lifetime: _serde::export::PhantomData<&'de ()>,
                        }
                        impl <'de> _serde::de::Visitor<'de> for __Visitor<'de>
                         {
                            type
                            Value
                            =
                            TestCounterEntity;
                            fn expecting(&self,
                                         __formatter:
                                             &mut _serde::export::Formatter)
                             -> _serde::export::fmt::Result {
                                _serde::export::Formatter::write_str(__formatter,
                                                                     "struct TestCounterEntity")
                            }
                            #[inline]
                            fn visit_seq<__A>(self, mut __seq: __A)
                             ->
                                 _serde::export::Result<Self::Value,
                                                        __A::Error> where
                             __A: _serde::de::SeqAccess<'de> {
                                let __field0 =
                                    match match _serde::de::SeqAccess::next_element::<i32>(&mut __seq)
                                              {
                                              _serde::export::Ok(__val) =>
                                              __val,
                                              _serde::export::Err(__err) => {
                                                  return _serde::export::Err(__err);
                                              }
                                          } {
                                        _serde::export::Some(__value) =>
                                        __value,
                                        _serde::export::None => {
                                            return _serde::export::Err(_serde::de::Error::invalid_length(0usize,
                                                                                                         &"struct TestCounterEntity with 1 element"));
                                        }
                                    };
                                _serde::export::Ok(TestCounterEntity{counter:
                                                                         __field0,})
                            }
                            #[inline]
                            fn visit_map<__A>(self, mut __map: __A)
                             ->
                                 _serde::export::Result<Self::Value,
                                                        __A::Error> where
                             __A: _serde::de::MapAccess<'de> {
                                let mut __field0:
                                        _serde::export::Option<i32> =
                                    _serde::export::None;
                                while let _serde::export::Some(__key) =
                                          match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                              {
                                              _serde::export::Ok(__val) =>
                                              __val,
                                              _serde::export::Err(__err) => {
                                                  return _serde::export::Err(__err);
                                              }
                                          } {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::export::Option::is_some(&__field0)
                                               {
                                                return _serde::export::Err(<__A::Error
                                                                               as
                                                                               _serde::de::Error>::duplicate_field("counter"));
                                            }
                                            __field0 =
                                                _serde::export::Some(match _serde::de::MapAccess::next_value::<i32>(&mut __map)
                                                                         {
                                                                         _serde::export::Ok(__val)
                                                                         =>
                                                                         __val,
                                                                         _serde::export::Err(__err)
                                                                         => {
                                                                             return _serde::export::Err(__err);
                                                                         }
                                                                     });
                                        }
                                        _ => {
                                            let _ =
                                                match _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)
                                                    {
                                                    _serde::export::Ok(__val)
                                                    => __val,
                                                    _serde::export::Err(__err)
                                                    => {
                                                        return _serde::export::Err(__err);
                                                    }
                                                };
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        _serde::export::Some(__field0) =>
                                        __field0,
                                        _serde::export::None =>
                                        match _serde::private::de::missing_field("counter")
                                            {
                                            _serde::export::Ok(__val) =>
                                            __val,
                                            _serde::export::Err(__err) => {
                                                return _serde::export::Err(__err);
                                            }
                                        },
                                    };
                                _serde::export::Ok(TestCounterEntity{counter:
                                                                         __field0,})
                            }
                        }
                        const FIELDS: &'static [&'static str] = &["counter"];
                        _serde::Deserializer::deserialize_struct(__deserializer,
                                                                 "TestCounterEntity",
                                                                 FIELDS,
                                                                 __Visitor{marker:
                                                                               _serde::export::PhantomData::<TestCounterEntity>,
                                                                           lifetime:
                                                                               _serde::export::PhantomData,})
                    }
                }
            };
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::fmt::Debug for TestCounterEntity {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
             -> ::std::fmt::Result {
                match *self {
                    TestCounterEntity { counter: ref __self_0_0 } => {
                        let mut debug_trait_builder =
                            f.debug_struct("TestCounterEntity");
                        let _ =
                            debug_trait_builder.field("counter",
                                                      &&(*__self_0_0));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::marker::Copy for TestCounterEntity { }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::clone::Clone for TestCounterEntity {
            #[inline]
            fn clone(&self) -> TestCounterEntity {
                { let _: ::std::clone::AssertParamIsClone<i32>; *self }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::cmp::PartialEq for TestCounterEntity {
            #[inline]
            fn eq(&self, other: &TestCounterEntity) -> bool {
                match *other {
                    TestCounterEntity { counter: ref __self_1_0 } =>
                    match *self {
                        TestCounterEntity { counter: ref __self_0_0 } =>
                        (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &TestCounterEntity) -> bool {
                match *other {
                    TestCounterEntity { counter: ref __self_1_0 } =>
                    match *self {
                        TestCounterEntity { counter: ref __self_0_0 } =>
                        (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl Default for TestCounterEntity {
            fn default() -> Self { Self{counter: 0,} }
        }
        impl Aggregator<TestEvents, String, PgQuery> for TestCounterEntity {
            fn apply_event(acc: Self, event: &TestEvents) -> Self {
                let counter =
                    match event {
                        TestEvents::Inc(ref inc) =>
                        acc.counter + inc.data.num,
                    };
                Self{counter, ..acc}
            }
            fn query(_query_args: String) -> PgQuery {
                let params: Vec<Box<ToSql + Send + Sync>> = Vec::new();
                PgQuery::new("select * from events", params)
            }
        }
        impl EventHandler for TestEvent {
            fn handle_event(event: Event<Self>, _store: &Store) {
                {
                    let lvl = ::log::Level::Trace;
                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                           lvl <= ::log::max_level() {
                        ::log::__private_api_log(::std::fmt::Arguments::new_v1(&["TestEvent handler "],
                                                                               &match (&event,)
                                                                                    {
                                                                                    (arg0,)
                                                                                    =>
                                                                                    [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                 ::std::fmt::Debug::fmt)],
                                                                                }),
                                                 lvl,
                                                 &("event_store::internals::test_helpers",
                                                   "event_store::internals::test_helpers",
                                                   "event-store/src/internals/test_helpers.rs",
                                                   76u32));
                    }
                };
            }
        }
        fn current_time_ms() -> u64 {
            let start = SystemTime::now();
            let since_the_epoch =
                start.duration_since(UNIX_EPOCH).expect("Time went backwards");
            let in_ms =
                since_the_epoch.as_secs() * 1000 +
                    since_the_epoch.subsec_nanos() as u64 / 1000000;
            in_ms
        }
        /// Create a new database with a random name, returning the connection
        pub fn pg_create_random_db(suffix: Option<&str>)
         -> Pool<PostgresConnectionManager> {
            let db_id =
                ::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["eventstorerust-",
                                                                     "-"],
                                                                   &match (&current_time_ms(),
                                                                           &suffix.unwrap_or("test"))
                                                                        {
                                                                        (arg0,
                                                                         arg1)
                                                                        =>
                                                                        [::std::fmt::ArgumentV1::new(arg0,
                                                                                                     ::std::fmt::Display::fmt),
                                                                         ::std::fmt::ArgumentV1::new(arg1,
                                                                                                     ::std::fmt::Display::fmt)],
                                                                    }));
            {
                ::std::io::_print(::std::fmt::Arguments::new_v1(&["Create test DB ",
                                                                  "\n"],
                                                                &match (&db_id,)
                                                                     {
                                                                     (arg0,)
                                                                     =>
                                                                     [::std::fmt::ArgumentV1::new(arg0,
                                                                                                  ::std::fmt::Display::fmt)],
                                                                 }));
            };
            let manager =
                PostgresConnectionManager::new("postgres://postgres@localhost:5430",
                                               TlsMode::None).unwrap();
            let pool = r2d2::Pool::new(manager).unwrap();
            let conn = pool.get().unwrap();
            conn.batch_execute(&::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["CREATE DATABASE \"",
                                                                                     "\""],
                                                                                   &match (&db_id,)
                                                                                        {
                                                                                        (arg0,)
                                                                                        =>
                                                                                        [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                     ::std::fmt::Display::fmt)],
                                                                                    }))).unwrap();
            let manager =
                PostgresConnectionManager::new(::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["postgres://postgres@localhost:5430/"],
                                                                                                  &match (&db_id,)
                                                                                                       {
                                                                                                       (arg0,)
                                                                                                       =>
                                                                                                       [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                    ::std::fmt::Display::fmt)],
                                                                                                   })),
                                               TlsMode::None).unwrap();
            let pool = r2d2::Pool::new(manager).unwrap();
            pool
        }
    }
    use futures::Future as OldFuture;
    use std::future::Future as NewFuture;
    pub fn backward<I, E>(f: impl NewFuture<Output = Result<I, E>>)
     -> impl OldFuture<Item = I, Error = E> {
        use tokio_async_await::compat::backward;
        backward::Compat::new(f)
    }
    pub fn forward<I, E>(f: impl OldFuture<Item = I, Error = E> + Unpin)
     -> impl NewFuture<Output = Result<I, E>> {
        use tokio_async_await::compat::forward::IntoAwaitable;
        f.into_awaitable()
    }
}
pub mod prelude {
    //! Event store prelude
    pub use crate::aggregator::Aggregator;
    pub use crate::event::Event;
    pub use crate::event_context::EventContext;
    pub use crate::event_handler::EventHandler;
    pub use crate::store::Store;
    pub use crate::store_query::StoreQuery;
    pub use crate::subscribe_options::SubscribeOptions;
}
pub use crate::aggregator::Aggregator;
pub use crate::event::Event;
pub use crate::event_context::EventContext;
pub use crate::event_handler::EventHandler;
pub use crate::store::Store;
pub use crate::store_query::StoreQuery;
pub use crate::subscribable_store::SubscribableStore;
pub use crate::subscribe_options::SubscribeOptions;
pub use event_store_derive_internals::{EventData, Events};
