# Event store explorer

Postico, but for the event store

## Setup/running

* Install the GTK development libraries (from <https://gtk-rs.org/docs-src/requirements.html>):

    ```bash
    # Debian and Ubuntu
    sudo apt install libgtk-3-dev

    # Fedora
    sudo dnf install gtk3-devel glib2-devel

    # Fedora 21 and earlier
    sudo yum install gtk3-devel glib2-devel

    # OS X
    brew install gtk+3
    ```
* Ensure you have the `cmp` environment running locally, or another Postgres container listening locally on `postgres://repositive:repositive@localhost:5432` and a RabbitMQ container listening at `127.0.0.1:5672` (default `guest`/`guest`/ credentials).
* Run the SQL snippet described in [Timestamps](#timestamps) to condition the database.
* Run with `cargo run -- <database> <event namespace> <event type>`, e.g. `cargo run -- organisations organisations PolicyUpdated`.
* (optional) Debug with `RUST_LOG=level cargo run ...`

## Timestamps

The dates stored in the `time` column is somewhat incorrect. Rust's `chrono` crate parses dates using the RFC3339 standard, a stricter subset of ISO8601. This requires an update to the database to store the timestamp in a format that Rust can understand:

```sql
update events
set context = context || jsonb_build_object(
    'time',
    replace(
        replace(((context->>'time')::timestamp with time zone)::text, '+00', 'Z'),
        ' ', 'T'
    )
)
```

The above snippet normalises existing timestamps in the database to the strict `2019-01-17T19:53:44.339Z` format. Javascript's `(new Date()).toISOString()` produces strings like `"2019-01-17T19:53:44.339Z"`, which are valid (`Z` is equivalent to `+00:00`).
