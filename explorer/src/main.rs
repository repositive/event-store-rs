// enable the await! macro, async support, and the new std::Futures api.
#![feature(await_macro, async_await, futures_api)]
// only needed to manually implement a std future:
#![feature(arbitrary_self_types)]

use event_store::{
    adapters::{AmqpEmitterAdapter, PgCacheAdapter, PgStoreAdapter},
    SubscribableStore,
};
use gtk::prelude::*;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use std::io;
use std::net::SocketAddr;

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
    let manager =
        PostgresConnectionManager::new("postgres://repositive:repositive@localhost:5432/organisations", TlsMode::None)
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

    tokio::run_async(
        async {
            let _store = await!(create_store());

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

            results_store.insert_with_values(None, &[0, 1, 2], &[&"ONE", &"Super", &"nice"]);
            results_store.insert_with_values(None, &[0, 1, 2], &[&"TWO", &"Super", &"nice"]);

            window.show_all();

            gtk::main();
        },
    );
}
