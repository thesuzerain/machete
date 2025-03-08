CREATE TABLE encounter_types (
    id SERIAL PRIMARY KEY,
    name text NOT NULL
);
INSERT INTO encounter_types (id, name) VALUES 
(0, 'Unknown'),
(1, 'Reward Initialization'),
(2, 'Reward'),
(3, 'Combat'),
(4, 'Subsystem');

CREATE TABLE encounter_type_subsystem_types (
    id SERIAL PRIMARY KEY,
    name text NOT NULL
);
INSERT INTO encounter_type_subsystem_types (id, name) VALUES 
(0, 'Unknown'),
(1, 'Chase'),
(2, 'Infiltration'),
(3, 'Research');

ALTER TABLE encounters ADD COLUMN encounter_type_id integer NOT NULL REFERENCES encounter_types(id);
ALTER TABLE encounters ADD COLUMN subsystem_type_id integer REFERENCES encounter_type_subsystem_types(id);

CREATE TABLE encounter_skill_checks (
    id SERIAL PRIMARY KEY,
    order_index integer NOT NULL,
    encounter_id integer NOT NULL REFERENCES encounters(id),
    name text NOT NULL,
    vp integer NOT NULL
);
CREATE INDEX encounter_skill_checks_encounter_id_index ON encounter_skill_checks(encounter_id);

CREATE TABLE encounter_skill_check_rolls (
    encounter_skill_check_id integer NOT NULL REFERENCES encounter_skill_checks(id),
    roll VARCHAR(16) NOT NULL,
    dc integer NOT NULL
);
CREATE INDEX encounter_skill_check_rolls_encounter_skill_check_id_index ON encounter_skill_check_rolls(encounter_skill_check_id);