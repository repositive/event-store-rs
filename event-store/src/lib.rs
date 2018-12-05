#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate event_store_derive;
#[macro_use]
extern crate log;

pub mod aggregator;
pub mod amqp;
pub mod event;
pub mod event_context;
pub mod event_saver;
pub mod pg;
pub mod store_query;
#[doc(hidden)]
pub mod test_helpers;

pub use crate::aggregator::*;
pub use crate::amqp::*;
pub use crate::event::Event;
pub use crate::event_saver::*;
pub use crate::pg::*;
pub use crate::store_query::*;
#[doc(hidden)]
pub use crate::test_helpers::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
