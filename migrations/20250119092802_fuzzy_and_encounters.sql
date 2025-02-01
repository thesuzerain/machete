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
    play_date timestamptz NOT NULL,

    -- Non-metadata or unique. A collection of unassigned rewards in linked encounters.
    -- eg: The gold in a session is the sum of all the gold in the encounters, minus the gold assigned to characters.
    unassigned_gold_rewards double precision NOT NULL DEFAULT 0,
    unassigned_item_rewards INTEGER[] NOT NULL DEFAULT '{}'
);

ALTER TABLE encounters ADD COLUMN session_id INTEGER 0 REFERENCES campaign_sessions(id);

-- Gold is entirely fungible, and items are more-or-less fungible, so we can just assign them to characters directly on a session-by-session basis,
-- and mostly ignore what encounters they came from. It does mean that the uniqueness of an item is lost, but currently items are non-unique anyway.
-- If unique items are added (eg: history, runes, etc), we'll need a different id system for them anyway to uniquely identify them, meaning we can still ignore the encounter they came from.
CREATE TABLE campaign_session_characters (
    session_id INTEGER NOT NULL REFERENCES campaign_sessions(id),
    character_id INTEGER NOT NULL REFERENCES characters(id),

    gold_rewards double precision NOT NULL DEFAULT 0,
    item_rewards INTEGER[] NOT NULL DEFAULT '{}',

    PRIMARY KEY (session_id, character_id)
);

ALTER TABLE event_groups ADD COLUMN session_id INTEGER NOT NULL DEFAULT 0 REFERENCES campaign_sessions(id);
ALTER TABLE events ADD COLUMN session_id INTEGER NOT NULL DEFAULT 0 REFERENCES campaign_sessions(id);

ALTER TABLE event_groups ADD COLUMN intra_session_order INTEGER NOT NULL DEFAULT 0;
ALTER TABLE events ADD COLUMN intra_session_order INTEGER NOT NULL DEFAULT 0;

