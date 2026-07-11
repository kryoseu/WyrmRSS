-- Feed icons live in a separate table so the blob never gets pulled with
-- the feed rows every list query selects. 
CREATE TABLE feed_icons (
    feed_id      INTEGER PRIMARY KEY REFERENCES feeds(id) ON DELETE CASCADE,
    data         BYTEA,
    content_type TEXT,
    checked_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
