ALTER TABLE posts ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
UPDATE posts SET created_at = published_at;
CREATE INDEX idx_posts_created_at ON posts (created_at DESC, id DESC);
DROP INDEX idx_posts_published_at;

CREATE INDEX idx_post_archive_archived_at ON post_archive (archived_at DESC, id DESC);
DROP INDEX idx_post_archive_published_at;
