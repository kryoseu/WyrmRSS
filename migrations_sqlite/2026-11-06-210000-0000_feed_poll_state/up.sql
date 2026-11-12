CREATE TABLE feed_poll_state (
    feed_id INTEGER NOT NULL PRIMARY KEY REFERENCES feeds(id) ON DELETE CASCADE,
    source_url TEXT NOT NULL,
    filter_signature TEXT NOT NULL,
    checked_at TIMESTAMP NOT NULL,
    last_error TEXT,
    etag TEXT,
    last_modified TEXT
);
