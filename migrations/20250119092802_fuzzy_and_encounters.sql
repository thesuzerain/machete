CREATE EXTENSION pg_trgm;

ALTER TABLE encounters ADD COLUMN extra_experience INTEGER NOT NULL DEFAULT 0;
ALTER TABLE encounters ADD COLUMN total_experience INTEGER NOT NULL DEFAULT 0;
ALTER TABLE encounters ADD COLUMN total_treasure_value double precision NOT NULL DEFAULT 0;
ALTER TABLE encounters ADD COLUMN enemy_level_adjustments SMALLINT[] NOT NULL DEFAULT '{}'; -- Assumed to be alongside 'enemies' array

ALTER TABLE encounters ALTER COLUMN treasure_currency TYPE double precision;
ALTER TABLE library_items ALTER COLUMN price TYPE double precision;

CREATE TABLE campaign_sessions (
    id SERIAL PRIMARY KEY,
    session_order INTEGER NOT NULL,
    campaign_id INTEGER NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    play_date timestamptz NOT NULL
);

ALTER TABLE encounters ADD COLUMN session_id INTEGER 0 REFERENCES campaign_sessions(id);

ALTER TABLE event_groups ADD COLUMN session_id INTEGER NOT NULL DEFAULT 0 REFERENCES campaign_sessions(id);
ALTER TABLE events ADD COLUMN session_id INTEGER NOT NULL DEFAULT 0 REFERENCES campaign_sessions(id);

ALTER TABLE event_groups ADD COLUMN intra_session_order INTEGER NOT NULL DEFAULT 0;
ALTER TABLE events ADD COLUMN intra_session_order INTEGER NOT NULL DEFAULT 0;