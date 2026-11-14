CREATE TABLE sessions (
    token_hash TEXT NOT NULL PRIMARY KEY,
    expires_at TIMESTAMP NOT NULL,
    last_seen_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_sessions_expires_at ON sessions (expires_at);
