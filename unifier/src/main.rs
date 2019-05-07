#[macro_use]
extern crate log;
#[macro_use]
extern crate structopt;

mod config;
mod event;

use config::{Config, ConfigConnection};
use event::Event;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
#[structopt(name = "unify", about = "Unify multiple event stores into one")]
struct Options {
    /// Which connection to select from connections.toml
    #[structopt(short = "c", long = "connection")]
    connection: String,

    /// Truncate destination events table before inserting data
    #[structopt(long = "truncate-dest")]
    truncate_dest: bool,
}

fn collect_domain_events(
    domain: &str,
    namespace: &str,
    connection: &ConfigConnection,
) -> Result<Vec<Event>, String> {
    info!(
        "Collecting events from domain {} where namespace is {}...",
        domain, namespace
    );

    let mut conn = Client::connect(&format!("{}/{}", connection.source_db_uri, domain), NoTls)
        .map_err(|e| e.to_string())?;

    // NOTE: This query reformats dates to be RFC3339 compatible
    conn.query(
        r#"select
            id,
            data,
            context || jsonb_build_object('time', to_timestamp(context->>'time', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"')) as context
        from events where data->>'event_namespace' = $1 order by context->>'time' asc"#,
        &[&namespace],
    )
    .map_err(|e| e.to_string())
    .map(|result| {
        info!("Collected {} events for namespace {} in domain {}", result.len(), namespace, domain);

        result.iter().map(|row| {
            let id: Uuid = row.get(0);
            let data: serde_json::Value = row.get(1);
            let context: serde_json::Value = row.get(2);

            Event {
                id,
                data: serde_json::from_value(data).expect(&format!("Failed to parse event data for event ID {} (domain {})", id, domain)),
                context: serde_json::from_value(context).expect(&format!("Failed to parse event context for event ID {} (domain {})", id, domain)),
            }
        }).collect()
    })
}

fn main() -> Result<(), String> {
    pretty_env_logger::init();

    let config = Config::load()?;

    let args = Options::from_args();

    debug!("Args {:?}", args);

    let connection = config.get(&args.connection).expect(&format!(
        "Failed to find config for key {}",
        args.connection
    ));

    debug!("Connection {:?}", connection);

    let domain_events: Vec<Vec<Event>> = connection
        .domains
        .iter()
        .map(|(domain, namespace)| collect_domain_events(domain, namespace, connection).unwrap())
        .collect();

    let domain_events_sum: usize = domain_events.iter().map(|events| events.len()).sum();

    debug!("Collected total {} events", domain_events_sum);

    let all_events: HashMap<Uuid, Event> = domain_events
        .into_iter()
        .flat_map(|events| events.into_iter().map(|event| (event.id, event)))
        .collect();

    if all_events.len() != domain_events_sum {
        error!("Unique list of events is not the same length as all collected events");
        error!(
            "Expected all events {} to equal domain events {}",
            all_events.len(),
            domain_events_sum
        );

        return Err(String::from("Events are not properly unique"));
    }

    let mut dest_connection =
        Client::connect(&connection.dest_db_uri.clone(), NoTls).map_err(|e| e.to_string())?;

    let mut txn = dest_connection.transaction().map_err(|e| e.to_string())?;
    let stmt = txn
        .prepare("insert into events (id, data, context) values ($1, $2, $3)")
        .map_err(|e| e.to_string())?;

    if args.truncate_dest {
        warn!("Truncating destination table events");

        txn.simple_query("truncate table events")
            .map_err(|e| e.to_string())?;
    }

    for (id, event) in all_events.iter() {
        debug!("Insert event {}", id);

        txn.execute(
            &stmt,
            &[
                &event.id,
                &serde_json::to_value(&event.data).unwrap(),
                &serde_json::to_value(&event.context).unwrap(),
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    txn.commit().map_err(|e| e.to_string())?;

    info!("Successfully inserted {} events", all_events.len());

    Ok(())
}
