CREATE TABLE feeds (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    url TEXT NOT NULL UNIQUE,
    ttl INTEGER NOT NULL DEFAULT 900,
    url_filter TEXT[] NOT NULL DEFAULT '{}',
    last_fetched_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
