-- Instance-level configuration (not UI settings).
CREATE TABLE local_instance (
    is_singleton BOOLEAN NOT NULL PRIMARY KEY DEFAULT TRUE,
    is_setup     BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT only_one_row CHECK (is_singleton = TRUE)
);

-- The desktop is single-user and never shows setup.
INSERT INTO local_instance (is_singleton, is_setup) VALUES (TRUE, TRUE);
