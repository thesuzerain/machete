CREATE TABLE users (
    id INTEGER PRIMARY KEY
);

CREATE TABLE library_objects (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(64) NOT NULL,
    game_system INT NOT NULL,
    owner BIGINT REFERENCES users
);

CREATE TABLE library_items (
    id INTEGER PRIMARY KEY REFERENCES library_objects,
    rarity INT,
    level INT,
    price INT
);

CREATE TABLE library_creatures (
    id INTEGER PRIMARY KEY REFERENCES library_objects,
    rarity INT,
    level INT,
    alignment INT,
    size INT
);

CREATE TABLE library_spells (
    id INTEGER PRIMARY KEY REFERENCES library_objects,
    rarity INT,
    rank INT
);

CREATE TABLE library_objects_tags (
    library_object_id INTEGER REFERENCES library_objects,
    tag_id INT REFERENCES tags
);
CREATE TABLE tags (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    tag VARCHAR(8) NOT NULL
);