CREATE DATABASE eventstorerust;

\c eventstorerust

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

DROP TABLE events;

CREATE TABLE events (
    id uuid DEFAULT uuid_generate_v4() PRIMARY KEY,
    data jsonb NOT NULL,
    context jsonb DEFAULT '{}'::jsonb
);

INSERT INTO "events"("data","context")
VALUES
(E'{"type": "eventstore.Increment", "test_field": "inc_dec", "by": 1}',E'{"time": "2018-03-02T00:00:00+00"}'),
(E'{"type": "eventstore.Decrement", "test_field": "inc_dec", "by": 1}',E'{"time": "2018-03-02T01:00:00+00"}'),
(E'{"type": "eventstore.Increment", "test_field": "inc_dec", "by": 2}',E'{"time": "2018-03-02T02:00:00+00"}'),
(E'{"type": "eventstore.Other", "test_field": "other", "foo": "bar"}',E'{"time": "2018-03-02T04:00:00+00"}'),
(E'{"type": "eventstore.Unknown"}',E'{"time": "2018-03-02T05:00:00+00"}')
;
