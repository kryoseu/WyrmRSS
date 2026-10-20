CREATE TABLE host_credentials (
    id         INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    host       TEXT NOT NULL UNIQUE,
    kind       TEXT NOT NULL
               CHECK (kind IN ('basic_auth', 'custom_header')),
    name       TEXT NOT NULL,
    secret     TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
