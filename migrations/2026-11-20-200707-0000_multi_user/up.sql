CREATE TABLE local_users (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT,
    oidc_subject TEXT UNIQUE,
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Existing single-user data belongs to the first (admin) user
INSERT INTO local_users (username, is_admin) VALUES ('admin', TRUE);

ALTER TABLE feeds            ADD COLUMN user_id INT NOT NULL DEFAULT 1 REFERENCES local_users(id) ON DELETE CASCADE;
ALTER TABLE folders          ADD COLUMN user_id INT NOT NULL DEFAULT 1 REFERENCES local_users(id) ON DELETE CASCADE;
ALTER TABLE webhooks         ADD COLUMN user_id INT NOT NULL DEFAULT 1 REFERENCES local_users(id) ON DELETE CASCADE;
ALTER TABLE host_credentials ADD COLUMN user_id INT NOT NULL DEFAULT 1 REFERENCES local_users(id) ON DELETE CASCADE;
ALTER TABLE post_archive     ADD COLUMN user_id INT NOT NULL DEFAULT 1 REFERENCES local_users(id) ON DELETE CASCADE;
ALTER TABLE sessions         ADD COLUMN user_id INT NOT NULL DEFAULT 1 REFERENCES local_users(id) ON DELETE CASCADE;
ALTER TABLE shortcut_overrides ADD COLUMN user_id INT NOT NULL DEFAULT 1 REFERENCES local_users(id) ON DELETE CASCADE;

-- The default above is temporary to allow setting non-nullable user_id;
-- dropping it prevents an insert without user_id from silently going to admin.
ALTER TABLE feeds              ALTER COLUMN user_id DROP DEFAULT;
ALTER TABLE folders            ALTER COLUMN user_id DROP DEFAULT;
ALTER TABLE webhooks           ALTER COLUMN user_id DROP DEFAULT;
ALTER TABLE host_credentials   ALTER COLUMN user_id DROP DEFAULT;
ALTER TABLE post_archive       ALTER COLUMN user_id DROP DEFAULT;
ALTER TABLE sessions           ALTER COLUMN user_id DROP DEFAULT;
ALTER TABLE shortcut_overrides ALTER COLUMN user_id DROP DEFAULT;

ALTER TABLE shortcut_overrides DROP CONSTRAINT shortcut_overrides_pkey,
    ADD PRIMARY KEY (user_id, shortcut_action);

-- Needed because these were unique instance-wide, so a user couldn't add a feed URL, folder
-- name, host or shortcut key another user already has. Leading with user_id also indexes it.
ALTER TABLE feeds DROP CONSTRAINT feeds_url_key,
    ADD CONSTRAINT feeds_user_url_key UNIQUE (user_id, url);
ALTER TABLE host_credentials DROP CONSTRAINT host_credentials_host_key,
    ADD CONSTRAINT host_credentials_user_host_key UNIQUE (user_id, host);
ALTER TABLE shortcut_overrides DROP CONSTRAINT shortcut_overrides_key_key,
    ADD CONSTRAINT shortcut_overrides_user_key_key UNIQUE (user_id, key);
DROP INDEX idx_folders_name_lower;
CREATE UNIQUE INDEX idx_folders_user_name_lower ON folders (user_id, LOWER(name));

CREATE INDEX idx_webhooks_user_id ON webhooks (user_id);
CREATE INDEX idx_post_archive_user_id ON post_archive (user_id);
