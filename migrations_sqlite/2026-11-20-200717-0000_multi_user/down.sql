-- Indexes first: SQLite can't drop a column that an index uses.
DROP INDEX idx_host_credentials_user_host;
DROP INDEX idx_folders_user_name_lower;
DROP INDEX idx_feeds_user_url;

CREATE TABLE shortcut_overrides_old (
    shortcut_action TEXT NOT NULL PRIMARY KEY,
    key             TEXT NOT NULL UNIQUE
);
INSERT INTO shortcut_overrides_old (shortcut_action, key)
    SELECT shortcut_action, key FROM shortcut_overrides;
DROP TABLE shortcut_overrides;
ALTER TABLE shortcut_overrides_old RENAME TO shortcut_overrides;

ALTER TABLE feeds            DROP COLUMN user_id;
ALTER TABLE folders          DROP COLUMN user_id;
ALTER TABLE webhooks         DROP COLUMN user_id;
ALTER TABLE host_credentials DROP COLUMN user_id;
ALTER TABLE post_archive     DROP COLUMN user_id;
ALTER TABLE sessions         DROP COLUMN user_id;

DROP TABLE users;
