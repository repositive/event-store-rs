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
