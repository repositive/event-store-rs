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

// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

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

fn connect_copy_button(source_label: &gtk::Label, button: &gtk::Button, window: &gtk::Window) {
    button.connect_clicked(
        clone!(source_label, window => move |_button| {
            if let Some(value) = source_label.get_label() {
                let clip = window.get_clipboard(&gdk::ATOM_NONE);

                clip.set_text(&value);
            }
        }),
    );
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

            for evt in test_events.iter().cloned() {
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

            // --- Search input

            let query_input: gtk::Entry = builder.get_object("query-input").expect("query-input");

            query_input.connect_activate(|input| {
                if let Some(value) = input.get_text() {
                    info!("Search for {}", value);
                }
            });

            // ---

            // --- Display events on click

            let selected_event_id_label: gtk::Label = builder
                .get_object("current-event-id")
                .expect("current-event-id");
            let selected_event_id_copy: gtk::Button = builder
                .get_object("copy-current-event-id")
                .expect("copy-current-event-id");

            let selected_event_namespace_label: gtk::Label = builder
                .get_object("current-event-namespace")
                .expect("current-event-namespace");
            let selected_event_namespace_copy: gtk::Button = builder
                .get_object("copy-current-event-namespace")
                .expect("copy-current-event-namespace");

            let selected_event_type_label: gtk::Label = builder
                .get_object("current-event-type")
                .expect("current-event-type");
            let selected_event_type_copy: gtk::Button = builder
                .get_object("copy-current-event-type")
                .expect("copy-current-event-type");

            connect_copy_button(&selected_event_id_label, &selected_event_id_copy, &window);
            connect_copy_button(&selected_event_type_label, &selected_event_type_copy, &window);
            connect_copy_button(&selected_event_namespace_label, &selected_event_namespace_copy, &window);

            let selected_event_data: gtk::TextView = builder
                .get_object("selected-event-data")
                .expect("selected-event-data");
            let selected_event_context: gtk::TextView = builder
                .get_object("selected-event-context")
                .expect("selected-event-context");

            let event_data_buf = gtk::TextBuffer::new(None);
            let event_context_buf = gtk::TextBuffer::new(None);

            selected_event_data.set_buffer(Some(&event_data_buf));
            selected_event_context.set_buffer(Some(&event_context_buf));

            let selected_result = results_list.get_selection();

            selected_result.connect_changed(clone!(test_events => move |_selection| {
                let (_model, path) = _selection.get_selected().expect("Could not get selected");

                let selected_uuid: Uuid = results_store
                    .get_value(&path, 0)
                    .get::<&str>()
                    .and_then(|uuid_str| Uuid::parse_str(uuid_str).ok())
                    .expect("Could not parse value into UUID");


                let selected = test_events.iter().find(|evt| evt.id == selected_uuid).expect(&format!("Could not event with UUID {}", selected_uuid));

                trace!("Selected event {:?}", selected);

                selected_event_id_label.set_label(&selected_uuid.to_string());
                selected_event_namespace_label.set_label(&selected.data["event_namespace"].as_str().unwrap_or("(no namespace)"));
                selected_event_type_label.set_label(&selected.data["event_type"].as_str().unwrap_or("(no type)"));

                event_data_buf.set_text(&serde_json::to_string_pretty(&selected.data).unwrap());
                event_context_buf.set_text(&serde_json::to_string_pretty(&selected.context).unwrap());
            }));

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
