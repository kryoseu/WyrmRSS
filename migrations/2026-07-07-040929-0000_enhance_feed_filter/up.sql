ALTER TABLE feeds RENAME COLUMN url_filter TO filters;

-- Pre-rename filters only ever matched URLs; scope them explicitly so the
-- broadened matching (bare pattern = any field) doesn't change their behavior.
UPDATE feeds SET filters = ARRAY(SELECT 'url:' || f FROM unnest(filters) AS f);
