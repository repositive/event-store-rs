// enable the await! macro, async support, and the new std::Futures api.
#![feature(await_macro, async_await, futures_api)]
// only needed to manually implement a std future:
#![feature(arbitrary_self_types)]

#[macro_use]
mod macros;

use chrono::prelude::*;
use event_store::{
    adapters::{AmqpEmitterAdapter, PgCacheAdapter, PgStoreAdapter},
    internals::backward,
    EventContext, SubscribableStore,
};
use gtk::prelude::*;
use log::{debug, info, trace};
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use serde_derive::Deserialize;
use serde_json::Value as JsonValue;
use std::io;
use std::net::SocketAddr;
// use structopt::StructOpt;
use tokio::runtime::current_thread::Runtime as CurrentThreadRuntime;
use uuid::Uuid;

enum ResultColumn {
    Id = 0,
    Data = 1,
    Context = 2,
}

// #[derive(StructOpt, Debug)]
// #[structopt(name = "explorer")]
// struct CliOpts {
//     /// The database use. Connects to postgres://repositive:repositive@localhost:5432/<DATABASE>
//     #[structopt(name = "DATABASE")]
//     database: String,

//     /// The event namespace, e.g. `organisations` or `analysis`
//     #[structopt(name = "NAMESPACE")]
//     event_namespace: String,

//     /// Event type, e.g. `PolicyUpdated`, `AccountCreated`
//     #[structopt(name = "EVENT")]
//     event_type: String,
// }

#[derive(Deserialize, Debug, Clone)]
struct AnyEvent {
    id: Uuid,
    data: JsonValue,
    context: EventContext,
}

fn window() -> (gtk::Window, gtk::Builder) {
    let glade_src = include_str!("glade/main.glade");
    let builder = gtk::Builder::new_from_string(glade_src);

    let window: gtk::Window = builder.get_object("window-main").expect("window-main");

    (window, builder)
}

fn add_column(list: &gtk::TreeView, ty: &str, title: &str, idx: ResultColumn) {
    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    column.set_fixed_width(250);
    column.set_resizable(true);
    column.pack_start(&cell, true);
    column.set_title(title);
    column.add_attribute(&cell, ty, idx as i32);
    list.append_column(&column);
}

fn connect_copy_button(source_label: &gtk::Label, button: &gtk::Button, window: &gtk::Window) {
    button.connect_clicked(clone!(source_label, window => move |_button| {
        if let Some(value) = source_label.get_label() {
            let clip = window.get_clipboard(&gdk::ATOM_NONE);

            clip.set_text(&value);
        }
    }));
}

fn connect(db: &String) -> Result<Pool<PostgresConnectionManager>, io::Error> {
    let manager = PostgresConnectionManager::new(
        format!("postgres://repositive:repositive@localhost:5432/{}", db),
        TlsMode::None,
    )?;

    let pool = r2d2::Pool::new(manager)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(pool)
}

async fn create_store(
    pool: &Pool<PostgresConnectionManager>,
) -> Result<SubscribableStore, io::Error> {
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

async fn do_search(query: String, store: &SubscribableStore) -> Result<Vec<AnyEvent>, io::Error> {
    let parts: Vec<&str> = query.split('.').collect();

    let forever = Utc.ymd(1970, 1, 1).and_hms(0, 0, 0);

    await!(store
        .internals_get_store()
        .read_events_since(parts[0], parts[1], forever))
    .map(|result| {
        result
            .into_iter()
            .map(|res| {
                trace!("Event ID {}", res["id"]);

                let evt: AnyEvent = serde_json::from_value(res).expect("Failed to parse");

                evt
            })
            .collect::<Vec<AnyEvent>>()
    })
}

fn get_databases(pool: &Pool<PostgresConnectionManager>) -> Result<Vec<String>, io::Error> {
    let result = pool.get().unwrap().query(
        "select datname as database from pg_database where datistemplate = false;",
        &[],
    )?;

    let names = result.iter().map(|row| row.get(0)).collect();

    debug!("Found databases: {:?}", names);

    Ok(names)
}

fn populate_results_store(results: &Vec<AnyEvent>, results_store: &gtk::ListStore) {
    results_store.clear();

    for evt in results.iter() {
        trace!("Event {:?}", evt);

        results_store.insert_with_values(
            None,
            &[
                ResultColumn::Id as u32,
                ResultColumn::Data as u32,
                ResultColumn::Context as u32,
            ],
            &[
                &format!("{}", evt.id),
                &format!("{}", evt.data),
                &format!("{}", serde_json::to_string(&evt.context).unwrap()),
            ],
        );
    }
}

fn populate_databases_chooser(
    pool: &Pool<PostgresConnectionManager>,
    builder: &gtk::Builder,
) -> gtk::ComboBox {
    let databases = get_databases(&pool).expect("Could not fetch list of databases");

    let dropdown: gtk::ComboBoxText = builder
        .get_object("database-chooser")
        .expect("database-chooser");

    for d in databases {
        dropdown.append_text(&d);
    }

    dropdown.upcast::<gtk::ComboBox>()
}

fn main() {
    pretty_env_logger::init();

    // let opts = CliOpts::from_args();

    let pool = connect(&"organisations".to_string()).expect("Failed to connect");

    // debug!("{:?}", opts);

    let store = block!(create_store(&pool)).expect("Could not get store");

    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let (window, builder) = window();

    // --- Database picker

    let db_dropdown: gtk::ComboBox = populate_databases_chooser(&pool, &builder);

    db_dropdown.connect_changed(|combo| {
        // TODO: Fix clone
        let active_text = combo
            .clone()
            .downcast::<gtk::ComboBoxText>()
            .expect("Downcast failed")
            .get_active_text();

        if let Some(selected_value) = active_text {
            debug!("DB choosed: {:?}", selected_value);

            // TODO: Change DB
        }
    });

    // ---

    let results_list: gtk::TreeView = builder.get_object("list-results").expect("list-results");
    let results_store =
        gtk::ListStore::new(&[gtk::Type::String, gtk::Type::String, gtk::Type::String]);

    add_column(&results_list, "text", "ID", ResultColumn::Id);
    add_column(&results_list, "text", "Data", ResultColumn::Data);
    add_column(&results_list, "text", "Context", ResultColumn::Context);

    results_list.set_headers_visible(true);
    results_list.set_model(Some(&results_store));

    // --- Search input

    let query_input: gtk::Entry = builder.get_object("query-input").expect("query-input");

    query_input.connect_activate(clone!(results_store => move |input| {
        if let Some(value) = input.get_text() {
            info!("Search for {}", value);

            let items = value.split(".").collect::<Vec<&str>>();

            let results = block!(do_search(
                [items[0], items[1]].join("."),
                &store
            ))
            .expect("Search failed");

            populate_results_store(&results, &results_store);
        }
    }));

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
    connect_copy_button(
        &selected_event_type_label,
        &selected_event_type_copy,
        &window,
    );
    connect_copy_button(
        &selected_event_namespace_label,
        &selected_event_namespace_copy,
        &window,
    );

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

    selected_result.connect_changed(move |selection| {
        let (_model, path) = selection.get_selected().expect("Could not get selected");

        let uuid: Uuid = results_store
            .get_value(&path, ResultColumn::Id as i32)
            .get::<&str>()
            .and_then(|uuid_str| Uuid::parse_str(uuid_str).ok())
            .expect("Could not parse value into UUID");

        let data: JsonValue = results_store
            .get_value(&path, ResultColumn::Data as i32)
            .get::<&str>()
            .and_then(|data| serde_json::from_str(data).ok())
            .expect("Could not parse data");

        let context: JsonValue = results_store
            .get_value(&path, ResultColumn::Context as i32)
            .get::<&str>()
            .and_then(|context| serde_json::from_str(context).ok())
            .expect("Could not parse context");

        trace!("Selected event ID {:?}", uuid);

        selected_event_id_label.set_label(&uuid.to_string());
        selected_event_namespace_label
            .set_label(&data["event_namespace"].as_str().unwrap_or("(no namespace)"));
        selected_event_type_label.set_label(&data["event_type"].as_str().unwrap_or("(no type)"));

        event_data_buf.set_text(&serde_json::to_string_pretty(&data).unwrap());
        event_context_buf.set_text(&serde_json::to_string_pretty(&context).unwrap());
    });

    // ---

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}
