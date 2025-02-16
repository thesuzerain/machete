-- Extends library items with some more information useful for generating expected treasure
ALTER TABLE library_items ALTER COLUMN price TYPE double precision;
ALTER TABLE library_items ADD COLUMN item_categories varchar(64)[] NOT NULL DEFAULT '{}';
ALTER TABLE library_items ADD COLUMN consumable boolean NOT NULL DEFAULT FALSE;
ALTER TABLE library_items ADD COLUMN magical boolean NOT NULL DEFAULT FALSE;
ALTER TABLE library_items ADD COLUMN item_type varchar(16) ;
ALTER TABLE library_items ADD COLUMN apex_stat varchar(27) ;

ALTER TABLE library_creatures ADD COLUMN traits varchar(64)[] NOT NULL DEFAULT '{}';
ALTER TABLE library_spells ADD COLUMN traits varchar(64)[] NOT NULL DEFAULT '{}';
ALTER TABLE library_items ADD COLUMN traits varchar(64)[] NOT NULL DEFAULT '{}';

ALTER TABLE library_objects ADD COLUMN legacy boolean NOT NULL DEFAULT FALSE;

-- Alternate item for pre/post remastering. If something was renamed, this connects them.
ALTER TABLE library_objects ADD COLUMN remastering_alt_id int REFERENCES library_objects;

CREATE TABLE library_items_skill_boosts (
    item_id INT REFERENCES library_items(id),
    skill VARCHAR(64), -- Null if unrecognized
    bonus SMALLINT NOT NULL
);
CREATE TABLE stat_boost_category_types(
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);
INSERT INTO stat_boost_category_types(id, name)
VALUES 
    (1, 'Attack potency'), (2, 'Defense potency'), (3,'Devastating attacks'),  (4,'Skill potency'), 
    (5, 'Saving throw potency'), (6,'Perception potency'), (7,'Attribute apex');


CREATE TABLE runes (
    id SERIAL PRIMARY KEY,
    item_id INT NOT NULL REFERENCES library_items(id), -- Item of the rune
    name VARCHAR(64) NOT NULL,
    fundamental boolean NOT NULL DEFAULT FALSE,
    stat_boost_category_id SMALLINT REFERENCES stat_boost_category_types(id),
    legacy boolean NOT NULL DEFAULT FALSE,
    potency SMALLINT NOT NULL, -- May be +x, or x dice, etc. For property runes, this is the level of the rune
    applied_to_item_type VARCHAR(16) NOT NULL, -- What rune can be applied to

    UNIQUE(item_id, name),
    UNIQUE(name, legacy, potency, applied_to_item_type)
);
CREATE TABLE library_items_runes (
    item_id INT NOT NULL REFERENCES library_items(id),
    rune_id INT NOT NULL REFERENCES runes(id)
);


-- Expected treasure tables
CREATE TABLE expected_treasures_by_level(
    level SMALLINT PRIMARY KEY,
    total_value double precision NOT NULL,

    party_currency double precision NOT NULL,
    currency_per_additional_player double precision NOT NULL,

    -- number -> amount
    permanent_items_by_level jsonb NOT NULL,
    consumable_items_by_level jsonb NOT NULL,

    -- recommended per encounter
    encounter_low double precision NOT NULL,
    encounter_moderate double precision NOT NULL,
    encounter_severe double precision NOT NULL,
    encounter_extreme double precision NOT NULL,
    encounter_extra double precision NOT NULL
);

INSERT INTO expected_treasures_by_level(level, total_value, party_currency, currency_per_additional_player, permanent_items_by_level, consumable_items_by_level, encounter_low, encounter_moderate, encounter_severe, encounter_extreme, encounter_extra)
VALUES 
    (1, 175,    40,     10,     '{"2": 2, "1": 2}', '{"2": 2, "1": 3}',                 13, 18, 26, 35, 35),
    (2, 300,    70,     18,     '{"3": 2, "2": 2}', '{"3": 2, "2": 2, "1": 2}',         23, 30, 45, 60, 60),
    (3, 500,    120,    30,     '{"4": 2, "3": 2}', '{"4": 2, "3": 2, "2": 2}',         38, 50, 75, 100, 100),
    (4, 850,    200,    50,     '{"5": 2, "4": 2}', '{"5": 2, "4": 2, "3": 2}',         65, 85, 130, 170, 170),
    (5, 1350,   320,    80,     '{"6": 2, "5": 2}', '{"6": 2, "5": 2, "4": 2}',         100, 135, 200, 270, 270),
    (6, 2000,   500,    125,    '{"7": 2, "6": 2}', '{"7": 2, "6": 2, "5": 2}',         150, 200, 300, 400, 400),
    (7, 2900,   720,    180,     '{"8": 2, "7": 2}', '{"8": 2, "7": 2, "6": 2}',        220, 290, 440, 580, 580),
    (8, 4000,   1000,   250,    '{"9": 2, "8": 2}', '{"9": 2, "8": 2, "7": 2}',         300, 400, 600, 800, 800),
    (9, 5700,   1400,   350,    '{"10": 2, "9": 2}', '{"10": 2, "9": 2, "8": 2}',       430, 570, 860, 1140, 1140),
    (10,8000,   2000,   500,    '{"11": 2, "10": 2}', '{"11": 2, "10": 2, "9": 2}',     600, 800, 1200, 1600, 1600),
    (11,11500,  2800,   700,    '{"12": 2, "11": 2}', '{"12": 2, "11": 2, "10": 2}',    865, 1150, 1725, 2300, 2300),
    (12,16500,  4000,   1000,   '{"13": 2, "12": 2}', '{"13": 2, "12": 2, "11": 2}',    1250, 1650, 2475, 3300, 3300),
    (13,25000,  6000,   1500,   '{"14": 2, "13": 2}', '{"14": 2, "13": 2, "12": 2}',    1875, 2500, 3750, 5000, 5000),
    (14,36500,  9000,   2250,   '{"15": 2, "14": 2}', '{"15": 2, "14": 2, "13": 2}',    2750, 3650, 5500, 7300, 7300),
    (15,54500,  13000,  3250,   '{"16": 2, "15": 2}', '{"16": 2, "15": 2, "14": 2}',    4100, 5450, 8200, 10900, 10900),
    (16,82500,  20000,  5000,   '{"17": 2, "16": 2}', '{"17": 2, "16": 2, "15": 2}',    6200, 8250, 12400, 16500, 16500),
    (17,128000, 30000,  7500,   '{"18": 2, "17": 2}', '{"18": 2, "17": 2, "16": 2}',    9600, 12800, 19200, 25600, 25600),
    (18,208000, 48000,  12000,  '{"19": 2, "18": 2}', '{"19": 2, "18": 2, "17": 2}',    15600, 20800, 31200, 41600, 41600),
    (19,355000, 80000,  20000,  '{"20": 2, "19": 2}', '{"20": 2, "19": 2, "18": 2}',    26600, 35500, 53250, 71000, 71000),
    (20,490000, 140000, 35000,  '{"20": 4}', '{"20": 4, "19": 2}',                      36800, 49000, 73500, 98000, 98000);


-- Uses the 'automatic bonus progression' variant rule from Pathfinder 2e (which gives stat increases instead of items)
-- to approximate what the abilities of the items should be for characters of a given level
CREATE TABLE expected_treasure_stats_boosts_at_levels(
    level SMALLINT,
    stat_boost_category_id SMALLINT NOT NULL,
    amount SMALLINT NOT NULL
);

	
INSERT INTO expected_treasure_stats_boosts_at_levels(level, stat_boost_category_id, amount)
VALUES 
    (2, 1, 1), -- Attack potency +1
    (3, 4, 1), -- Skill potency (one at +1)
    (4, 3, 2), -- Devastating attacks (two dice)
    (5, 2, 1), -- Defense potency +1
    (6, 4, 1), (6, 4, 1), -- Skill potency (two at +1 each)
    (7, 6, 1), -- Perception potency +1
    (8, 5, 1), -- Saving throw potency +1
    (9, 4, 2), (9, 4, 1), -- Skill potency (one at +2, one at +1)
    (10, 1, 2), -- Attack potency +2
    (11, 2, 2), -- Defense potency +2
    (12, 3, 3), -- Devastating attacks (three dice)
    (13, 6, 2), (13, 4, 2), (13, 4, 1), -- Perception potency +2; skill potency (two at +2 each, one at +1)
    (14, 5, 2), -- Saving throw potency +2
    (15, 4, 2), (15, 4, 2), (15, 4, 2), (15, 4, 1), -- Skill potency (three at +2 each, one at +1)
    (16, 1, 3), -- Attack potency +3
    (17, 7, 1), (17, 4, 3), (17, 4, 2), (17, 4, 2), (17, 4, 1), -- Attribute apex; skill potency (one at +3, two at +2 each, two at +1 each)
    (18, 2, 3), -- Defense potency +3
    (19, 3, 4), (19, 6, 3), -- Devastating attacks (four dice), Perception potency +3
    (20, 5, 3), (20, 4, 3), (20, 4, 3), (20, 4, 2), (20, 4, 2), (20, 4, 1), (20, 4, 1); -- Saving throw potency +3; skill potency (two at +3 each, two at +2 each, two at +1 each)

-- For certain classes, we may want a certain specific item or a specific kind of item.
-- For example, wizards want spells.
CREATE TABLE expected_treasure_by_class_by_level(
    item_group_id SERIAL PRIMARY KEY,
    class_id SMALLINT NOT NULL,
    level SMALLINT NOT NULL,
    importance double precision NOT NULL -- 0-1, 0 = not important, 1 = very important
);
CREATE TABLE expected_treasure_by_class_by_level_traits(
    item_group_id SMALLINT NOT NULL,
    trait_id SMALLINT NOT NULL
);
CREATE TABLE expected_treasure_by_class_by_level_items(
    item_group_id SMALLINT NOT NULL,
    item_id SMALLINT NOT NULL
);