CREATE TYPE host_credential_kind AS ENUM ('basic_auth', 'custom_header');

CREATE TABLE host_credentials (
    id         SERIAL PRIMARY KEY,
    host       TEXT NOT NULL UNIQUE,
    kind       host_credential_kind NOT NULL,
    name       TEXT NOT NULL,
    secret     TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
