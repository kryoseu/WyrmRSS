ALTER TABLE post_archive ADD COLUMN tag TEXT;
ALTER TABLE post_archive ADD COLUMN tag_color TEXT;

ALTER TABLE feeds ADD COLUMN tag TEXT;
ALTER TABLE feeds ADD COLUMN tag_color TEXT;

UPDATE feeds
SET tag = folders.name
FROM folders
WHERE feeds.folder_id = folders.id;

ALTER TABLE feeds DROP COLUMN folder_id;

DROP TABLE folders;
