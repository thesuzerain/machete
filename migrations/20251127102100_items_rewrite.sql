CREATE TABLE campaign_items (
    id SERIAL PRIMARY KEY,
    library_item_id INT NOT NULL REFERENCES library_items(id),
    campaign_id INT NOT NULL REFERENCES campaigns(id),

    parent_item_id INT REFERENCES campaign_items(id),

    encounter_id INT REFERENCES encounters(id), -- reward for an encounter
    character_id INT REFERENCES characters(id), -- assigned to a character
    session_id INT REFERENCES campaign_sessions(id), -- rewarded in a session

    is_reward boolean NOT NULL DEFAULT FALSE, -- if true, should be counted towards budget, etc

    nickname VARCHAR(64),
    quantity SMALLINT NOT NULL DEFAULT 1,
    notes TEXT
);

CREATE INDEX idx_campaign_items_library_item_id ON campaign_items(library_item_id);
CREATE INDEX idx_campaign_items_campaign_id ON campaign_items(campaign_id);
CREATE INDEX idx_campaign_items_parent_item_id ON campaign_items(parent_item_id);


-- INSERT INTO campaign_items (library_item_id, campaign_id, 
--     parent_item_id, encounter_id, character_id, session_id, is_reward, nickname, quantity, notes)
-- SELECT 
--     unnest(cs.unassigned_item_rewards) as library_item_id,
--     cs.campaign_id,
--     NULL as parent_item_id,

-- FROM campaign_sessions cs


INSERT INTO campaign_items (library_item_id, campaign_id, 
    parent_item_id, encounter_id, character_id, session_id, is_reward, nickname, quantity, notes)
SELECT 

    
FROM encounter_treasure_items eti
INNER JOIN campaign_sessions cs
