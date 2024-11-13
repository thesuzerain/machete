ALTER TABLE users ADD COLUMN name VARCHAR(20);
INSERT INTO users(id, name) VALUES (0, 'Machete');
INSERT INTO users(id, name) VALUES (1, 'Test');

CREATE TABLE campaigns (
    id SERIAL PRIMARY KEY,
    owner BIGINT REFERENCES users NOT NULL,
    name VARCHAR(60) NOT NULL
);

CREATE TABLE characters (
    id SERIAL PRIMARY KEY,
    name VARCHAR(60) NOT NULL,
    player VARCHAR(60),
    campaign BIGINT REFERENCES campaigns NOT NULL
);

CREATE TABLE event_groups (
    id SERIAL PRIMARY KEY,
    campaign BIGINT REFERENCES campaigns NOT NULL,
    name VARCHAR(60) NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    description TEXT
);

CREATE TABLE events (
    id SERIAL PRIMARY KEY,
    event_group BIGINT REFERENCES event_groups,
    campaign BIGINT REFERENCES campaigns NOT NULL,
    character BIGINT REFERENCES characters,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    event_data JSONB NOT NULL
);

CREATE TABLE encounters (
    id SERIAL PRIMARY KEY,
    status SMALLINT NOT NULL,
    name VARCHAR(60) NOT NULL,
    description TEXT,
    enemies BIGINT[] NOT NULL,
    hazards BIGINT[] NOT NULL,
    treasure_currency INTEGER,
    treasure_items BIGINT[] NOT NULL
);