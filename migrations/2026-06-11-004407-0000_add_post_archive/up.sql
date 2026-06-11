CREATE TABLE post_archive (
    id INTEGER PRIMARY KEY,
    title TEXT,
    url TEXT,
    authors TEXT,
    published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    description TEXT,
    content TEXT,
    tag TEXT,
    tag_color TEXT,
    archived_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_post_archive_published_at ON post_archive(published_at DESC);
CREATE INDEX idx_post_archive_title_trgm ON post_archive USING gin (title gin_trgm_ops);

ALTER TABLE posts ADD COLUMN is_archived BOOLEAN NOT NULL DEFAULT FALSE;

