-- Rolling back fails if two users have the same feed URL, folder name, host or
-- shortcut key, since the restored single-user constraints don't allow duplicates.
DROP INDEX idx_post_archive_user_id;
DROP INDEX idx_webhooks_user_id;

DROP INDEX idx_folders_user_name_lower;
CREATE UNIQUE INDEX idx_folders_name_lower ON folders (LOWER(name));
ALTER TABLE shortcut_overrides DROP CONSTRAINT shortcut_overrides_user_key_key,
    ADD CONSTRAINT shortcut_overrides_key_key UNIQUE (key);
ALTER TABLE host_credentials DROP CONSTRAINT host_credentials_user_host_key,
    ADD CONSTRAINT host_credentials_host_key UNIQUE (host);
ALTER TABLE feeds DROP CONSTRAINT feeds_user_url_key,
    ADD CONSTRAINT feeds_url_key UNIQUE (url);

ALTER TABLE shortcut_overrides DROP CONSTRAINT shortcut_overrides_pkey,
    ADD PRIMARY KEY (shortcut_action);

ALTER TABLE feeds              DROP COLUMN user_id;
ALTER TABLE folders            DROP COLUMN user_id;
ALTER TABLE webhooks           DROP COLUMN user_id;
ALTER TABLE host_credentials   DROP COLUMN user_id;
ALTER TABLE post_archive       DROP COLUMN user_id;
ALTER TABLE sessions           DROP COLUMN user_id;
ALTER TABLE shortcut_overrides DROP COLUMN user_id;

DROP TABLE local_users;
