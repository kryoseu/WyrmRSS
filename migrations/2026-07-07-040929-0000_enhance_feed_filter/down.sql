-- Strip the url: scope added by up.sql; title:/content: filters have no
-- pre-rename equivalent and are left as-is.
UPDATE feeds SET filters = ARRAY(SELECT regexp_replace(f, '^url:', '') FROM unnest(filters) AS f);

ALTER TABLE feeds RENAME COLUMN filters TO url_filter;
