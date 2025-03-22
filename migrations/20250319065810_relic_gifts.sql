ALTER TABLE library_items ADD COLUMN relic_gift_stage INT;
ALTER TABLE library_items ADD COLUMN cursed BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE library_items DROP CONSTRAINT library_items_id_fkey;