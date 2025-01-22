CREATE EXTENSION pg_trgm;

ALTER TABLE encounters ADD COLUMN extra_experience INTEGER NOT NULL DEFAULT 0;
ALTER TABLE encounters ADD COLUMN total_experience INTEGER NOT NULL DEFAULT 0;
ALTER TABLE encounters ADD COLUMN total_treasure_value double precision NOT NULL DEFAULT 0;
ALTER TABLE encounters ADD COLUMN enemy_level_adjustments SMALLINT[] NOT NULL DEFAULT '{}'; -- Assumed to be alongside 'enemies' array

ALTER TABLE encounters ALTER COLUMN treasure_currency TYPE double precision;
ALTER TABLE library_items ALTER COLUMN price TYPE double precision;