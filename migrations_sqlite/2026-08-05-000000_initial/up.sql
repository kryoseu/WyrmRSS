-- Initial schema for the SQLite (desktop) backend.
--
-- SQLite does not enforce foreign keys unless `PRAGMA foreign_keys = ON` is set
-- on every connection, which makes the ON DELETE clauses below silently inert
-- without it. Applied in the pool's connection setup hook -- setting it here
-- would not persist past the migration.

CREATE TABLE folders (
    id   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_folders_name_lower ON folders (LOWER(name));

CREATE TABLE feeds (
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL,
    url             TEXT NOT NULL UNIQUE,
    ttl             INTEGER NOT NULL DEFAULT 900,
    -- JSON array; '[]' rather than NULL so reads never special-case absence.
    filters         TEXT NOT NULL DEFAULT '[]',
    last_fetched_at TIMESTAMP,
    created_at      TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    folder_id       INTEGER REFERENCES folders(id) ON DELETE SET NULL,
    is_paused       BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE posts (
    id           INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    feed_id      INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    title        TEXT,
    url          TEXT,
    authors      TEXT,
    published_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at   TIMESTAMP,
    description  TEXT,
    content      TEXT,
    bookmarked   BOOLEAN NOT NULL DEFAULT FALSE,
    is_read      BOOLEAN NOT NULL DEFAULT FALSE,
    is_archived  BOOLEAN NOT NULL DEFAULT FALSE,
    created_at   TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CONSTRAINT unique_post_per_feed UNIQUE (feed_id, url)
);

CREATE INDEX idx_posts_feed_id ON posts(feed_id);
CREATE INDEX idx_posts_created_at ON posts (created_at DESC, id DESC);

-- id is assigned from the originating post's id, so no AUTOINCREMENT.
CREATE TABLE post_archive (
    id           INTEGER NOT NULL PRIMARY KEY,
    title        TEXT,
    url          TEXT,
    authors      TEXT,
    published_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    description  TEXT,
    content      TEXT,
    archived_at  TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_post_archive_archived_at ON post_archive (archived_at DESC, id DESC);

CREATE TABLE settings (
    is_singleton             BOOLEAN NOT NULL PRIMARY KEY DEFAULT TRUE,
    page_size                INTEGER NOT NULL DEFAULT 100,
    feed_poll_interval_secs  INTEGER NOT NULL DEFAULT 900,
    http_timeout             INTEGER NOT NULL DEFAULT 30,
    http_connect_timeout     INTEGER NOT NULL DEFAULT 10,
    http_retries             INTEGER NOT NULL DEFAULT 3,
    http_user_agent          TEXT,
    read_mode                TEXT NOT NULL DEFAULT 'on_open'
                             CHECK (read_mode IN ('on_open', 'manually', 'disabled')),
    expire_read_after_days   INTEGER,
    expire_unread_after_days INTEGER,
    CONSTRAINT only_one_row CHECK (is_singleton = TRUE)
);

-- No INSERT ... DEFAULT VALUES in SQLite, so name the PK explicitly.
INSERT INTO settings (is_singleton) VALUES (TRUE);

CREATE TABLE webhooks (
    id               INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name             TEXT NOT NULL,
    url              TEXT NOT NULL,
    kind             TEXT NOT NULL
                     CHECK (kind IN ('discord', 'slack', 'custom')),
    payload_template TEXT,
    created_at       TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE feed_webhooks (
    feed_id    INTEGER NOT NULL REFERENCES feeds(id)    ON DELETE CASCADE,
    webhook_id INTEGER NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    PRIMARY KEY (feed_id, webhook_id)
);

CREATE INDEX idx_feed_webhooks_webhook_id ON feed_webhooks(webhook_id);

-- Icons live apart from feeds so the blob never gets pulled with the feed rows
-- every list query selects.
CREATE TABLE feed_icons (
    feed_id      INTEGER NOT NULL PRIMARY KEY REFERENCES feeds(id) ON DELETE CASCADE,
    data         BLOB,
    content_type TEXT,
    checked_at   TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- History of expired (deleted) posts: the (feed_id, url) dedup constraint on
-- posts forgets a post the moment its row is deleted, so any expired post still
-- listed in the upstream feed would be re-inserted on the next poll.
CREATE TABLE expired_posts (
    feed_id    INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    url        TEXT NOT NULL,
    expired_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (feed_id, url)
);

-- The pg_trgm GIN indexes on posts.title, posts.description and
-- post_archive.title have no direct equivalent -- FTS5 is a different shape, not
-- a translation. Until it's added, substring search here falls back to an
-- unindexed LIKE scan.
