CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    title TEXT,
    url TEXT,
    authors TEXT,
    published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    description TEXT,
    content TEXT,
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT unique_post_per_feed UNIQUE (feed_id, url)
);

CREATE INDEX idx_posts_feed_id ON posts(feed_id);
CREATE INDEX idx_posts_published_at ON posts(published_at DESC);
CREATE INDEX idx_posts_title_trgm ON posts USING gin (title gin_trgm_ops);
