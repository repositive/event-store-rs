#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate event_store_derive;
#[macro_use]
extern crate log;

mod event;
mod event_context;

pub mod amqp;
pub mod pg;

pub use crate::amqp::*;
pub use crate::event::Event;
pub use crate::pg::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
