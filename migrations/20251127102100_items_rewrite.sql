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


-- INSERT INTO item_instances (library_item_id, campaign_id, 
--     parent_item_id, encounter_id, character_id, session_id, is_reward, nickname, quantity, notes)
-- SELECT 
--     unnest(cs.unassigned_item_rewards) as library_item_id,
--     cs.campaign_id,
--     NULL as parent_item_id,

-- FROM campaign_sessions cs


INSERT INTO item_instances (library_item_id, campaign_id, 
    parent_item_id, encounter_id, character_id, session_id, is_reward, nickname, quantity, notes)
SELECT 

    
FROM encounter_treasure_items eti
INNER JOIN campaign_sessions cs
