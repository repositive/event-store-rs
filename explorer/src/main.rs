// enable the await! macro, async support, and the new std::Futures api.
#![feature(await_macro, async_await, futures_api)]
// only needed to manually implement a std future:
#![feature(arbitrary_self_types)]

use chrono::prelude::*;
use event_store::{
    adapters::{AmqpEmitterAdapter, PgCacheAdapter, PgStoreAdapter},
    EventContext, SubscribableStore,
};
use gtk::prelude::*;
use log::{debug, info, trace};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use serde_derive::Deserialize;
use serde_json::Value as JsonValue;
use std::io;
use std::net::SocketAddr;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(StructOpt, Debug)]
#[structopt(name = "explorer")]
struct CliOpts {
    /// The database use. Connects to postgres://repositive:repositive@localhost:5432/<DATABASE>
    #[structopt(name = "DATABASE")]
    database: String,

    /// The event namespace, e.g. `organisations` or `analysis`
    #[structopt(name = "NAMESPACE")]
    event_namespace: String,

    /// Event type, e.g. `PolicyUpdated`, `AccountCreated`
    #[structopt(name = "EVENT")]
    event_type: String,
}

#[derive(Deserialize, Debug, Clone)]
struct AnyEvent {
    id: Uuid,
    data: JsonValue,
    context: EventContext,
}

fn add_column(list: &gtk::TreeView, ty: &str, title: &str, idx: i32) {
    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    column.set_fixed_width(250);
    column.set_resizable(true);
    column.pack_start(&cell, true);
    column.set_title(title);
    column.add_attribute(&cell, ty, idx);
    list.append_column(&column);
}

async fn create_store(db: &String) -> Result<SubscribableStore, io::Error> {
    let manager = PostgresConnectionManager::new(
        format!("postgres://repositive:repositive@localhost:5432/{}", db),
        TlsMode::None,
    )?;

    let pool = r2d2::Pool::new(manager).expect("Could not create pool");

    let addr: SocketAddr = "127.0.0.1:5672"
        .parse()
        .expect("Could not parse RabbitMQ address");

    let store_adapter = await!(PgStoreAdapter::new(pool.clone()))?;
    let cache_adapter = await!(PgCacheAdapter::new(pool.clone()))?;
    let emitter_adapter = await!(AmqpEmitterAdapter::new(
        addr,
        "iris".into(),
        "_explorer".into()
    ))?;

    await!(SubscribableStore::new(
        store_adapter,
        cache_adapter,
        emitter_adapter
    ))
}

fn main() {
    pretty_env_logger::init();

    tokio::run_async(
        async {
            let opts = CliOpts::from_args();

            debug!("{:?}", opts);

            let store = await!(create_store(&opts.database)).expect("Could not get store");

            let forever = Utc.ymd(1970, 1, 1).and_hms(0, 0, 0);

            let test_events = await!(store.internals_get_store().read_events_since(
                &opts.event_namespace,
                &opts.event_type,
                forever
            ))
            .expect("Could not create store")
            .into_iter()
            .map(|res| {
                trace!("Event ID {}", res["id"]);

                let evt: AnyEvent = serde_json::from_value(res).expect("Failed to parse");

                evt
            })
            .collect::<Vec<AnyEvent>>();

            info!("Collected {} events", test_events.len());

            if gtk::init().is_err() {
                println!("Failed to initialize GTK.");
                return;
            }
            let glade_src = include_str!("glade/main.glade");
            let builder = gtk::Builder::new_from_string(glade_src);

            let window: gtk::Window = builder.get_object("window-main").expect("window-main");

            let results_list: gtk::TreeView =
                builder.get_object("list-results").expect("list-results");
            let results_store =
                gtk::ListStore::new(&[gtk::Type::String, gtk::Type::String, gtk::Type::String]);

            add_column(&results_list, "text", "ID", 0);
            add_column(&results_list, "text", "Data", 1);
            add_column(&results_list, "text", "Context", 2);

            results_list.set_headers_visible(true);
            results_list.set_model(Some(&results_store));

            let first = test_events[0].clone();

            for evt in test_events {
                trace!("Event {:?}", evt);

                results_store.insert_with_values(
                    None,
                    &[0, 1, 2],
                    &[
                        &format!("{}", evt.id),
                        &format!("{}", evt.data),
                        &format!("{}", serde_json::to_string(&evt.context).unwrap()),
                    ],
                );
            }

            // --- Display events on click

            let selected_event_data: gtk::TextView = builder
                .get_object("selected-event-data")
                .expect("selected-event-data");
            let selected_event_context: gtk::TextView = builder
                .get_object("selected-event-context")
                .expect("selected-event-context");

            let event_data_buf = gtk::TextBuffer::new(None);
            let event_context_buf = gtk::TextBuffer::new(None);

            event_data_buf.set_text(&serde_json::to_string_pretty(&first.data).unwrap());
            event_context_buf.set_text(&serde_json::to_string_pretty(&first.context).unwrap());
            selected_event_data.set_buffer(Some(&event_data_buf));
            selected_event_context.set_buffer(Some(&event_context_buf));

            // ---

            window.show_all();

            window.connect_delete_event(|_, _| {
                gtk::main_quit();
                Inhibit(false)
            });

            gtk::main();
        },
    );
}
