CREATE INDEX IF NOT EXISTS idx_post_archive_published_at ON post_archive(published_at DESC);
DROP INDEX IF EXISTS idx_post_archive_archived_at;

CREATE INDEX IF NOT EXISTS idx_posts_published_at ON posts(published_at DESC);
DROP INDEX IF EXISTS idx_posts_created_at;
ALTER TABLE posts DROP COLUMN IF EXISTS created_at;
