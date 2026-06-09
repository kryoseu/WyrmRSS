CREATE TYPE read_mode AS ENUM ('on_open', 'manually', 'disabled');
ALTER TABLE settings ADD COLUMN read_mode read_mode NOT NULL DEFAULT 'on_open';
