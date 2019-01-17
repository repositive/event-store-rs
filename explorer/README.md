# Event store explorer

Postico, but for the event store

## Setup

Currently the explorer is hardcoded to talk to the CMP local dev environment's `organisations` database. Make sure that's running.

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
