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

CREATE TABLE events (
    id SERIAL PRIMARY KEY,
    campaign BIGINT REFERENCES campaigns,
    character BIGINT REFERENCES characters,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    event_data JSONB NOT NULL
);