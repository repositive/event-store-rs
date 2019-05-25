use event_store::{Event, EventStore, Events};
use postgres::{Client, NoTls};
use r2d2_postgres::PostgresConnectionManager;

struct ThingCreated {
    foo: String,
}

struct ThingUpdated {
    bar: u32,
}

// TODO: Support default namespace on enum
// TODO: Support default entity type on enum
// TODO: Support leaving out `event_type` attrib and gleaning it from `Event<D>`
#[derive(Events)]
enum ExampleEvents {
    // TODO: Validate casing of attributes
    #[event_store(
        event_namespace = "store",
        event_type = "ThingCreated",
        entity_type = "thing"
    )]
    ThingCreated(Event<ThingCreated>),

    #[event_store(
        event_namespace = "store",
        event_type = "ThingUpdated",
        entity_type = "thing"
    )]
    ThingUpdated(Event<ThingUpdated>),
}

fn main() {
    println!("Example");
}
