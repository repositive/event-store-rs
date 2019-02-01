use event_store::EventHandler;
use event_store_derive::*;
use log::trace;
use serde_derive::{Deserialize, Serialize};

#[test]
fn create_default_event_handler() {
    pretty_env_logger::init();

    #[derive(EventData)]
    #[event_store(namespace = "_test")]
    struct NoopHandler {
        pub some_field: String,
    };

    impl EventHandler for NoopHandler {};

    trace!("NoopHandler created and no error occured");
}
