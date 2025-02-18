ALTER TABLE users ADD COLUMN name VARCHAR(20);
INSERT INTO users(id, name) VALUES (0, 'Machete');

ALTER TABLE library_objects ADD COLUMN url VARCHAR(255);
ALTER TABLE library_objects ADD COLUMN description TEXT;

ALTER TABLE library_spells ADD COLUMN traditions VARCHAR(32)[] NOT NULL DEFAULT '{}';

CREATE TABLE library_classes (
    id INTEGER PRIMARY KEY REFERENCES library_objects,
    rarity INT NOT NULL,
    hp INT NOT NULL,
    traditions VARCHAR(32)[] NOT NULL DEFAULT '{}'
);

CREATE TABLE library_hazards (
    id INTEGER PRIMARY KEY REFERENCES library_objects,
    rarity INT,
    level INT
);

CREATE TABLE campaigns (
    id SERIAL PRIMARY KEY,
    owner INT REFERENCES users NOT NULL,
    name VARCHAR(60) NOT NULL,
    description TEXT
);

CREATE TABLE characters (
    id SERIAL PRIMARY KEY,
    name VARCHAR(60) NOT NULL,
    player VARCHAR(60),
    level INT NOT NULL,
    campaign INT REFERENCES campaigns NOT NULL,
    class INT REFERENCES library_classes NOT NULL
);

CREATE TABLE event_groups (
    id SERIAL PRIMARY KEY,
    campaign INT REFERENCES campaigns NOT NULL,
    name VARCHAR(60) NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    description TEXT
);

CREATE TABLE events (
    id SERIAL PRIMARY KEY,
    event_group INT REFERENCES event_groups,
    campaign INT REFERENCES campaigns NOT NULL,
    character INT REFERENCES characters,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    event_data JSONB NOT NULL
);

CREATE TABLE encounters (
    id SERIAL PRIMARY KEY,
    status SMALLINT NOT NULL,
    name VARCHAR(60) NOT NULL,
    description TEXT,
    party_level INT NOT NULL,
    party_size INT NOT NULL,
    treasure_currency INTEGER
);

CREATE TABLE encounter_enemies (
    encounter INT REFERENCES encounters,
    enemy INT NOT NULL REFERENCES library_creatures
);

CREATE TABLE encounter_hazards (
    encounter INT REFERENCES encounters,
    hazard INT NOT NULL REFERENCES library_hazards
);

CREATE TABLE encounter_treasure_items (
    encounter INT REFERENCES encounters,
    item INT NOT NULL REFERENCES library_items
);

CREATE INDEX ON encounter_enemies (encounter);
CREATE INDEX ON encounter_hazards (encounter);
CREATE INDEX ON encounter_treasure_items (encounter);