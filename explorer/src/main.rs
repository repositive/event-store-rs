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
use log::info;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use serde_derive::Deserialize;
use serde_json::Value as JsonValue;
use std::io;
use std::net::SocketAddr;
use uuid::Uuid;

#[derive(Deserialize)]
struct AnyEvent {
    id: Uuid,
    data: JsonValue,
    context: EventContext,
}

fn add_column(list: &gtk::TreeView, ty: &str, title: &str, idx: i32) {
    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    column.pack_start(&cell, true);
    column.set_title(title);
    // Association of the view's column with the model's `id` column.
    column.add_attribute(&cell, ty, idx);
    list.append_column(&column);
}

async fn create_store() -> Result<SubscribableStore, io::Error> {
    let manager = PostgresConnectionManager::new(
        "postgres://repositive:repositive@localhost:5432/organisations",
        TlsMode::None,
    )
    .unwrap();

    let pool = r2d2::Pool::new(manager).unwrap();

    let addr: SocketAddr = "127.0.0.1:5672".parse().unwrap();

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

    // MASSIVE issue with some timestamps currently in the DB: they're not valid! ISO8601 is
    // somewhat lax, but Chrono tries to parse as either RFC2822 or RFC3339. We could attempt a
    // custom parse with [parse_from_str](https://docs.rs/chrono/0.4.6/chrono/struct.DateTime.html#method.parse_from_str)
    // but why not just fix the DB so it's correct? I think most cases are just missing the last two
    // digits of a 4 digit timezone.
    // let test: DateTime<Utc> = "2019-01-16T22:07:04.845Z".parse().expect("Could not parse test 1");
    // let test2: DateTime<FixedOffset> = DateTime::parse_from_rfc2822("2018-10-10T19:52:21+00").expect("Could not parse test 2");
    // let test2: DateTime<FixedOffset> = DateTime::parse_from_rfc3339("2018-10-10T19:52:21+00").expect("Could not parse test 3");
    // let test3: DateTime<Utc> = "2018-10-10T19:52:21+00".parse().unwrap();

    tokio::run_async(
        async {
            let store = await!(create_store()).unwrap();

            let forever = Utc.ymd(1970, 1, 1).and_hms(0, 0, 0);

            let test_events = await!(store.internals_get_store().read_events_since(
                "organisations",
                "PolicyUpdated",
                forever
            ))
            .unwrap()
            .into_iter()
            .map(|res| {
                let evt: AnyEvent = serde_json::from_value(res).unwrap();

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

            let window: gtk::Window = builder.get_object("window-main").unwrap();

            let results_list: gtk::TreeView = builder.get_object("list-results").unwrap();
            let results_store =
                gtk::ListStore::new(&[gtk::Type::String, gtk::Type::String, gtk::Type::String]);

            add_column(&results_list, "text", "ID", 0);
            add_column(&results_list, "text", "Data", 1);
            add_column(&results_list, "text", "Context", 2);

            results_list.set_headers_visible(true);
            results_list.set_model(Some(&results_store));

            for evt in test_events {
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

            window.show_all();

            window.connect_delete_event(|_, _| {
                gtk::main_quit();
                Inhibit(false)
            });

            gtk::main();
        },
    );
}
