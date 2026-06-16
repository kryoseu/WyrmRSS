CREATE INDEX idx_posts_description_trgm ON posts USING gin (description gin_trgm_ops);
