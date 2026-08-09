CREATE TYPE display_mode AS ENUM ('river', 'feed_only', 'radar');

ALTER TABLE feeds ADD COLUMN display_mode display_mode NOT NULL DEFAULT 'river';

