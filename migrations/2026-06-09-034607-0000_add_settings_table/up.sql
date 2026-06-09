CREATE TABLE settings (
  is_singleton BOOL PRIMARY KEY DEFAULT TRUE,
  CONSTRAINT only_one_row CHECK (is_singleton = TRUE),
  page_size INTEGER NOT NULL DEFAULT 100,
  feed_poll_interval_secs INTEGER NOT NULL DEFAULT 900,
  http_timeout INTEGER NOT NULL DEFAULT 30,
  http_connect_timeout INTEGER NOT NULL DEFAULT 10,
  http_retries INTEGER NOT NULL DEFAULT 3,
  http_user_agent TEXT
);

INSERT INTO settings DEFAULT VALUES;
