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
(E'{"type": "some_namespace.Inc", "test_field": "inc_dec", "by": 1}',E'{"time": "2018-03-02T00:00:00+00"}'),
(E'{"type": "some_namespace.Dec", "test_field": "inc_dec", "by": 1}',E'{"time": "2018-03-02T01:00:00+00"}'),
(E'{"type": "some_namespace.Inc", "test_field": "inc_dec", "by": 2}',E'{"time": "2018-03-02T02:00:00+00"}')
-- (E'{"type": "some_namespace.Other", "test_field": "other", "foo": "bar"}',E'{"time": "2018-03-02T04:00:00+00"}'),
-- (E'{"type": "some_namespace.Unknown"}',E'{"time": "2018-03-02T05:00:00+00"}')
;
