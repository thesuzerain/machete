-- Temporary. Eventually, we will add encounter 'types' and this will be a type.
ALTER TABLE encounters ADD COLUMN initialization_encounter boolean NOT NULL DEFAULT false;