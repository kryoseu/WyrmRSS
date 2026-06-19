CREATE TYPE webhook_kind AS ENUM ('discord', 'slack', 'custom');

CREATE TABLE webhooks (
    id               SERIAL PRIMARY KEY,
    name             TEXT NOT NULL,
    url              TEXT NOT NULL,
    kind             webhook_kind NOT NULL,
    payload_template TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE feed_webhooks (
      feed_id    INTEGER NOT NULL REFERENCES feeds(id)    ON DELETE CASCADE,
      webhook_id INTEGER NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
      PRIMARY KEY (feed_id, webhook_id)
  );

CREATE INDEX idx_feed_webhooks_webhook_id ON feed_webhooks(webhook_id);
