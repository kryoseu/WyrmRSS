ALTER TABLE feeds ADD COLUMN display_mode TEXT NOT NULL DEFAULT 'river'
  CHECK (display_mode IN ('river', 'feed_only', 'radar'));
