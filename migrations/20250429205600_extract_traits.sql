ALTER TABLE tags RENAME TO library_tags;
ALTER TABLE library_tags ADD COLUMN trait boolean DEFAULT false;