-- History of expired (deleted) posts: the (feed_id, url) dedup constraint
-- on posts forgets a post the moment its row is deleted, so any expired post
-- still listed in the upstream feed would be re-inserted on the next poll.
CREATE TABLE expired_posts (
    feed_id    INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    url        TEXT NOT NULL,
    expired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (feed_id, url)
);
