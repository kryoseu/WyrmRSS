CREATE TABLE folders (
    id   SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_folders_name_lower ON folders (LOWER(name));

ALTER TABLE feeds ADD COLUMN folder_id INTEGER REFERENCES folders(id) ON DELETE SET NULL;

-- One folder per distinct tag (case/whitespace-insensitive);
-- casing taken from the most recently added feed using it.
INSERT INTO folders (name)
SELECT DISTINCT ON (LOWER(TRIM(tag))) TRIM(tag)
FROM feeds
WHERE tag IS NOT NULL AND TRIM(tag) <> ''
ORDER BY LOWER(TRIM(tag)), created_at DESC, id DESC;

UPDATE feeds
SET folder_id = folders.id
FROM folders
WHERE feeds.tag IS NOT NULL
  AND LOWER(TRIM(feeds.tag)) = LOWER(folders.name);

ALTER TABLE feeds DROP COLUMN tag;
ALTER TABLE feeds DROP COLUMN tag_color;

ALTER TABLE post_archive DROP COLUMN tag;
ALTER TABLE post_archive DROP COLUMN tag_color;
