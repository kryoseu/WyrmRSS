-- Instance-level configuration (not UI settings).
CREATE TABLE local_instance (
  is_singleton BOOL PRIMARY KEY DEFAULT TRUE,
  CONSTRAINT only_one_row CHECK (is_singleton = TRUE),
  is_setup BOOL NOT NULL DEFAULT FALSE
);

INSERT INTO local_instance DEFAULT VALUES;
