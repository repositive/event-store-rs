CREATE DATABASE eventstorerust;

\c eventstorerust

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

DROP TABLE events;

CREATE TABLE events (
    id uuid DEFAULT uuid_generate_v4() PRIMARY KEY,
    data jsonb NOT NULL,
    context jsonb DEFAULT '{}'::jsonb
);

DROP TABLE aggregate_cache;

CREATE TABLE aggregate_cache (
    id VARCHAR(64) PRIMARY KEY,
    -- id character varying(64) PRIMARY KEY,
    data jsonb NOT NULL,
    time timestamp without time zone DEFAULT now()
);

INSERT INTO "events"("data","context")
VALUES
(E'{"type": "some_namespace.Inc", "ident": "inc_dec", "by": 1}',E'{"time": "2018-03-02T00:00:00+00"}'),
(E'{"type": "some_namespace.Dec", "ident": "inc_dec", "by": 1}',E'{"time": "2018-03-02T01:00:00+00"}'),
(E'{"type": "some_namespace.Inc", "ident": "inc_dec", "by": 2}',E'{"time": "2018-03-02T02:00:00+00"}')
-- (E'{"type": "some_namespace.Other", "ident": "other", "foo": "bar"}',E'{"time": "2018-03-02T04:00:00+00"}'),
-- (E'{"type": "some_namespace.Unknown"}',E'{"time": "2018-03-02T05:00:00+00"}')
;
