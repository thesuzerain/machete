CREATE TABLE item_instances (
    id SERIAL PRIMARY KEY,
    library_item_id INT NOT NULL REFERENCES library_items(id),
    parent_item_id INT REFERENCES item_instances(id),

    campaign_id INT REFERENCES campaigns(id),
    encounter_id INT REFERENCES encounters(id), -- reward for an encounter
    character_id INT REFERENCES characters(id), -- assigned to a character
    session_id INT REFERENCES campaign_sessions(id), -- rewarded in a session

    is_reward boolean NOT NULL DEFAULT FALSE, -- if true, should be counted towards budget, etc

    nickname VARCHAR(64),
    quantity SMALLINT NOT NULL DEFAULT 1,
    notes TEXT
);

CREATE INDEX idx_item_instances_library_item_id ON item_instances(library_item_id);
CREATE INDEX idx_item_instances_campaign_id ON item_instances(campaign_id);
CREATE INDEX idx_item_instances_parent_item_id ON item_instances(parent_item_id);

INSERT INTO item_instances (library_item_id, campaign_id,
    parent_item_id, encounter_id, character_id, session_id, is_reward, nickname, quantity, notes)
SELECT
    eti.item as library_item_id,
    cs.campaign_id as campaign_id,
    null as parent_item_id,
    e.id as encounter_id,
    csci.character_id as character_id,
    cs.id as session_id,
    false is_reward,
    null as nickname,
    1 as quantity,
    null as notes

FROM (
    SELECT encounter, item, row_number() over (partition by encounter, item) r
    from encounter_treasure_items
) eti
INNER JOIN encounters e ON eti.encounter = e.id
LEFT JOIN campaign_sessions cs ON e.session_id = cs.id
LEFT JOIN (
    SELECT session_id, character_id, item_id, row_number() over (partition by session_id, character_id, item_id) r
    from campaign_session_character_items
) csci
    ON cs.id = csci.session_id
    AND csci.item_id = eti.item
    AND eti.r = csci.r;