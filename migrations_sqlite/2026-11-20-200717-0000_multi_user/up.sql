-- The desktop stays single-user; these columns exist so the shared database code
-- compiles against both backends. Everything belongs to the admin user (id 1).

CREATE TABLE local_users (
    id            INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT,
    oidc_subject  TEXT UNIQUE,
    is_admin      BOOLEAN NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

INSERT INTO local_users (username, is_admin) VALUES ('admin', TRUE);

-- No REFERENCES: SQLite can't drop a column that's part of a foreign key, which
-- would force table rebuilds in down.sql. The default stays since there's only one user.
ALTER TABLE feeds            ADD COLUMN user_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE folders          ADD COLUMN user_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE webhooks         ADD COLUMN user_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE host_credentials ADD COLUMN user_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE post_archive     ADD COLUMN user_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE sessions         ADD COLUMN user_id INTEGER NOT NULL DEFAULT 1;

-- Rebuilt because the primary key must match Postgres (user_id, shortcut_action)
-- for .find() to compile against both schemas. Nothing references this table.
-- user_id is the first column because print-schema lists key columns in column order.
CREATE TABLE shortcut_overrides_new (
    user_id         INTEGER NOT NULL DEFAULT 1,
    shortcut_action TEXT NOT NULL,
    key             TEXT NOT NULL,
    PRIMARY KEY (user_id, shortcut_action),
    UNIQUE (user_id, key)
);
INSERT INTO shortcut_overrides_new (shortcut_action, key)
    SELECT shortcut_action, key FROM shortcut_overrides;
DROP TABLE shortcut_overrides;
ALTER TABLE shortcut_overrides_new RENAME TO shortcut_overrides;

-- Needed because ON CONFLICT targets must match a unique index exactly, and the
-- shared upserts will target (user_id, ...). The old single-column ones are kept.
CREATE UNIQUE INDEX idx_feeds_user_url ON feeds (user_id, url);
CREATE UNIQUE INDEX idx_folders_user_name_lower ON folders (user_id, LOWER(name));
CREATE UNIQUE INDEX idx_host_credentials_user_host ON host_credentials (user_id, host);
