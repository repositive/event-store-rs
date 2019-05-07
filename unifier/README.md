# Unifier

This tool is to help unify multiple event store backing databases into a single table of all events for all domains.

## Getting started

* Get [rustup](https://rustup.rs)
* Create `connections.toml` in this folder
* Add a connection like this:
    ```toml
    [connections.local]
    source_db_uri = "postgres://repositive:repositive@localhost:5432"
    dest_db_uri = "postgres://repositive:repositive@localhost:5432/event-store"
    domains.accounts = "accounts"
    domains.analysis = "analysis"
    domains.bookmarks = "bookmarks"
    domains.cmp-inventory-overview = "cmp_overview"
    domains.mnemosyne = "metadata"
    domains.molecular-ingest = "molecular_ingest"
    domains.organisations = "organisations"
    domains.search = "gnowee"
    ```

    where `domains` is like `domains.<domain name> = <domain namespace>`
* `RUST_LOG=unifier=info cargo run --release -- --connection local `
* Run with `--release` for speedier operation
* Run with `--help` to see options

## Cleanup

A few operations are performed on events during unification:

* The legacy `data->>'type` field is removed. `data->>'event_namespace'` and `data->>'event_type'` should be used instead. Any new events created with the event store's `createEvent()` function already use these new fields.
* Timestamps are formatted to be valid RFC3339 timestamps, primarily to work well with Rust code. RFC3339 is a stricter subset of ISO8601. Any new events created with the event store's `createEvent()` function already produce valid dates.
* Any events where `context->>'subject'` is `null` are written using a default `{}` for `context->>'subject'` instead of `null`.
