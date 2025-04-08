use std::collections::HashMap;

use crate::models::encounter::EncounterType;
use crate::models::ids::InternalId;
use crate::models::stats::CampaignStats;
use crate::models::stats::{AssignedBoost, AssignedRewardsSession, CharacterStats, EncounterStats};

use serde::Deserialize;

pub async fn get_campaign_stats(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    campaign_id: InternalId,
) -> crate::Result<CampaignStats> {
    let characters_query = sqlx::query!(
        r#"
        SELECT
            ch.id,
            items.total_treasure_item_value,
            items.consumable_items,
            items.permanent_items,
            gold.total_gold,
            owned_boosts.assigned_boosts,
            expected_boosts.expected_boosts,
            reward_by_session.reward_by_session
        FROM characters ch
        INNER JOIN campaigns c ON ch.campaign = c.id
        LEFT JOIN LATERAL (
            SELECT
                SUM(csc.gold_rewards) AS total_gold
            FROM campaign_session_characters csc
            WHERE csc.character_id = ch.id

        ) gold ON true
        LEFT JOIN LATERAL (
            SELECT 
                ARRAY_AGG(item_id) filter (where consumable) consumable_items,
                ARRAY_AGG(item_id) filter (where NOT consumable) permanent_items,
                SUM(li.price) as total_treasure_item_value
            FROM campaign_session_character_items csci
            INNER JOIN campaign_sessions cs ON csci.session_id = cs.id
            INNER JOIN library_objects lo ON lo.id = csci.item_id
            INNER JOIN library_items li ON li.id = csci.item_id
            WHERE csci.character_id = ch.id
        ) items ON true
        LEFT JOIN LATERAL (
            SELECT
                JSONB_AGG(json_build_object(
                    'session_id', cs.id,
                    'treasure_gold', csc.gold_rewards,
                    'treasure_item_value', COALESCE(s.price_sum, 0),
                    'treasure_items_group', COALESCE(s.items_group, '{}')
                  ) ORDER BY cs.session_order) filter (WHERE cs.id is not null) as reward_by_session
            FROM  campaign_session_characters csc
            INNER JOIN campaign_sessions cs ON csc.session_id = cs.id
            LEFT JOIN LATERAL (
                SELECT SUM(li.price) as price_sum, ARRAY_AGG(csci.item_id) as items_group
                FROM campaign_session_character_items csci
                LEFT JOIN library_objects lo ON lo.id = csci.item_id
                LEFT JOIN library_items li ON li.id = csci.item_id
                WHERE csci.character_id = ch.id AND csci.session_id = cs.id
                GROUP BY csci.session_id
            ) s ON true
            WHERE csc.character_id = ch.id
        ) reward_by_session ON true
        LEFT JOIN LATERAL (
            SELECT json_agg(
                    json_build_object(
                    'boost_category_id', sbct.id,
                    'boost_category_name', sbct.name,
                    'potency', r.potency
                    )
                ) AS assigned_boosts
            FROM campaign_session_character_items csci
            INNER JOIN library_items li ON csci.item_id = li.id
            INNER JOIN library_objects lo ON li.id = lo.id
            INNER JOIN library_items_runes lir ON li.id = lir.item_id
            INNER JOIN runes r ON lir.rune_id = r.id
            INNER JOIN stat_boost_category_types sbct ON r.stat_boost_category_id = sbct.id
            WHERE csci.character_id = ch.id
        ) owned_boosts ON true
        LEFT JOIN LATERAL (
            SELECT json_agg(
                    json_build_object(
                    'boost_category_id', etsb.stat_boost_category_id,
                    'boost_category_name', sbct.name,
                    'potency', etsb.amount
                    )
                ) AS expected_boosts
                FROM expected_treasure_stats_boosts_at_levels etsb
                INNER JOIN stat_boost_category_types sbct ON etsb.stat_boost_category_id = sbct.id
            WHERE etsb.level <= c.level
        ) expected_boosts ON true
        WHERE c.owner = $1 AND c.id = $2
        "#,
        owner.0 as i32,
        campaign_id.0 as i32,
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|row| {
        let assigned_boosts: Vec<AssignedBoost> =
            serde_json::from_value(row.assigned_boosts.unwrap_or_default()).unwrap_or_default();
        let expected_boosts: Vec<AssignedBoost> =
            serde_json::from_value(row.expected_boosts.unwrap_or_default()).unwrap_or_default();

        let consumable_items: Vec<InternalId> = row
            .consumable_items
            .unwrap_or_default()
            .into_iter()
            .map(|i| InternalId(i as u32))
            .collect();
        let permanent_items: Vec<InternalId> = row
            .permanent_items
            .unwrap_or_default()
            .into_iter()
            .map(|i| InternalId(i as u32))
            .collect();

        let gold: f64 = row.total_gold.unwrap_or(0.0);
        let total_treasure_items_value: f64 = row.total_treasure_item_value.unwrap_or(0.0);

        #[derive(Deserialize, Debug)]
        pub struct AssignedRewardSession {
            pub session_id: u32,
            pub treasure_gold: f64,
            pub treasure_item_value: f64,
            pub treasure_items_group: Vec<u32>,
        }

        let rewards_by_session: Vec<AssignedRewardSession> =
            serde_json::from_value(row.reward_by_session.unwrap_or_default()).unwrap_or_default();
        let rewards_by_session = rewards_by_session
            .into_iter()
            .map(|r| AssignedRewardsSession {
                session_id: InternalId(r.session_id),
                treasure_gold: r.treasure_gold,
                treasure_item_value: r.treasure_item_value,
                treasure_items_group: r.treasure_items_group.into_iter().map(InternalId).collect(),
            })
            .collect();
        (
            InternalId(row.id as u32),
            CharacterStats {
                total_combined_treasure: gold + total_treasure_items_value,
                total_treasure_items_value,
                total_gold: gold,

                available_boosts: assigned_boosts,
                expected_boosts,

                rewards_per_session: rewards_by_session,

                total_permanent_items: permanent_items,
                total_consumable_items: consumable_items,
            },
        )
    })
    .collect::<HashMap<InternalId, CharacterStats>>();

    let query = sqlx::query!(
        r#"
        SELECT
            c.level,
            by_encounter.num_accomplishments,
            by_encounter.num_combat_encounters,
            by_encounter.num_subsystem_encounters,
            by_encounter.num_sessions,
            by_encounter.stats_by_encounter,
            by_encounter.total_item_treasure_value,
            by_encounter.total_treasure_currency_value,
            by_encounter.total_combined_treasure_value,
            items.total_treasure_items_value,
            by_encounter.total_experience,
            (by_encounter.total_experience % 1000) as experience_this_level,
            items_2.consumable_items_by_level,
            items_2.permanent_items_by_level,
            expected_consumable.expected_consumable_items_by_end_of_level,
            expected_permanent.expected_permanent_items_by_end_of_level,
            expected_combined_total_treasure_value_start_of_level,
            expected_combined_total_treasure_value_end_of_level
        FROM campaigns c
        LEFT JOIN LATERAL (
            SELECT
                SUM(e.total_items_value) AS total_item_treasure_value,
                SUM(e.treasure_currency) AS total_treasure_currency_value,
                SUM(e.total_items_value + e.treasure_currency) AS total_combined_treasure_value,
                SUM(e.total_experience) AS total_experience,
                JSONB_AGG(
                        json_build_object(
                                'session_id', cs.id,
                                'encounter_id', e.id,
                                'encounter_type_id', e.encounter_type_id,
                                'total_experience', e.total_experience,
                                'total_items_value', e.total_items_value,
                                'treasure_currency', e.treasure_currency,
                                'calculated_expected_total_treasure', ex.total_value * (e.total_experience / 1000.0),
                                'pf_expected_total_treasure', 
                                    CASE
                                        WHEN e.total_experience < 40 THEN ex.encounter_low
                                        WHEN e.total_experience < 80 THEN ex.encounter_moderate
                                        WHEN e.total_experience < 120 THEN ex.encounter_severe
                                        ELSE ex.encounter_extreme
                                    END
                        ) ORDER BY cs.session_order, cs.id, e.id -- TODO: Encounter ordering within a session?
                ) filter (WHERE e.id IS NOT NULL) as stats_by_encounter,
                COUNT(DISTINCT e.id) filter (WHERE e.encounter_type_id = 2) as num_accomplishments,
                COUNT(DISTINCT e.id) filter (WHERE e.encounter_type_id = 3) as num_combat_encounters,
                COUNT(DISTINCT e.id) filter (WHERE e.encounter_type_id = 4) as num_subsystem_encounters,
                COUNT(DISTINCT cs.id) as num_sessions
            FROM campaign_sessions cs
            LEFT JOIN encounters e ON e.session_id = cs.id
            INNER JOIN expected_treasures_by_level ex ON ex.level = c.level
            WHERE cs.campaign_id = c.id
        ) by_encounter ON true
        LEFT JOIN LATERAL (
            SELECT
                SUM(li.price) total_treasure_items_value
            FROM encounter_treasure_items eti
            INNER JOIN encounters e ON eti.encounter = e.id
            INNER JOIN campaign_sessions cs ON e.session_id = cs.id
            INNER JOIN library_objects lo ON lo.id = eti.item
            INNER JOIN library_items li ON li.id = eti.item
            WHERE cs.campaign_id = c.id
        ) items ON true
        LEFT JOIN LATERAL (
            SELECT
            jsonb_object_agg(level, total) FILTER (WHERE consumable) AS consumable_items_by_level,
            jsonb_object_agg(level, total) FILTER (WHERE NOT consumable) AS permanent_items_by_level
            FROM (
            SELECT
                li.level::text AS level,
                li.consumable,
                COUNT(*) AS total
            FROM encounter_treasure_items eti
            INNER JOIN encounters e ON eti.encounter = e.id
            INNER JOIN campaign_sessions cs ON e.session_id = cs.id
            INNER JOIN library_objects lo ON lo.id = eti.item
            INNER JOIN library_items li ON li.id = eti.item
            WHERE cs.campaign_id = 1
            GROUP BY li.level, li.consumable
            ) s
        ) items_2 ON true
        LEFT JOIN LATERAL (
            SELECT jsonb_object_agg(key, total) AS expected_consumable_items_by_end_of_level
            FROM (
            SELECT key, SUM(value::int) AS total
            FROM expected_treasures_by_level etbl,
                LATERAL jsonb_each(etbl.consumable_items_by_level)
            WHERE etbl.level <= c.level
            GROUP BY key
        ) s) expected_consumable ON true
        LEFT JOIN LATERAL (
                SELECT jsonb_object_agg(key, total) AS expected_permanent_items_by_end_of_level
                FROM (
                SELECT key, SUM(value::int) AS total
                FROM expected_treasures_by_level etbl,
                    LATERAL jsonb_each(etbl.permanent_items_by_level)
                WHERE etbl.level <= c.level
                GROUP BY key
        ) s) expected_permanent ON true
        LEFT JOIN LATERAL (
            SELECT
                SUM(total_value + charcount_diff*currency_per_additional_player) filter ( where etbl.level < c.level ) AS expected_combined_total_treasure_value_start_of_level, 
                SUM(total_value + charcount_diff*currency_per_additional_player) AS expected_combined_total_treasure_value_end_of_level
            FROM expected_treasures_by_level etbl,
            (
                SELECT COUNT(*)-4 AS charcount_diff FROM characters ch WHERE ch.campaign = c.id
            ) cd
            WHERE etbl.level <= c.level
        ) expected ON true
        WHERE c.owner = $1 AND c.id = $2    
        "#,
        owner.0 as i32,
        campaign_id.0 as i32,
    ).fetch_optional(exec).await?.map(|r| {
        let consumable_items_by_level : HashMap<u32, u32> = serde_json::from_value(r.consumable_items_by_level.unwrap_or_default()).unwrap_or_default();
        let permanent_items_by_level : HashMap<u32, u32> = serde_json::from_value(r.permanent_items_by_level.unwrap_or_default()).unwrap_or_default();
        let expected_consumable_items_by_end_of_level : HashMap<u32, u32> = serde_json::from_value(r.expected_consumable_items_by_end_of_level.unwrap_or_default()).unwrap_or_default();
        let expected_permanent_items_by_end_of_level : HashMap<u32, u32> = serde_json::from_value(r.expected_permanent_items_by_end_of_level.unwrap_or_default()).unwrap_or_default();

        #[derive(Deserialize, Debug)]
        pub struct OneEncounter {
            pub session_id: u32,
            pub encounter_id: u32,
            pub encounter_type_id: i32,
            pub total_experience:  i32,
            pub treasure_currency: f32,
            pub total_items_value: f32,
            pub calculated_expected_total_treasure: f32,
            pub pf_expected_total_treasure: f32,
        }
        let encounters : Vec<OneEncounter> = serde_json::from_value(r.stats_by_encounter.unwrap_or_default()).unwrap();
        let mut acc = 0;
        let encounters = encounters.into_iter().map(|e| {
            let stats = EncounterStats {
                encounter_ix: e.encounter_id,
                encounter_type: EncounterType::string_from_id(e.encounter_type_id),
                session_ix: acc,
                session_id: e.session_id,
                accumulated_items_treasure: e.total_items_value,
                accumulated_gold_treasure: e.treasure_currency,
                accumulated_xp: e.total_experience,
                calculated_expected_total_treasure: e.calculated_expected_total_treasure,
                pf_expected_total_treasure: e.pf_expected_total_treasure,
            };
            acc += 1;
            stats
        }).collect();

        let experience_this_level = r.experience_this_level.unwrap_or(0) as u32;
        let expected_combined_total_treasure_value_start_of_level = r.expected_combined_total_treasure_value_start_of_level.unwrap_or(0.0) as f32;
        let expected_combined_total_treasure_value_end_of_level = r.expected_combined_total_treasure_value_end_of_level.unwrap_or(0.0) as f32;

        let fraction_through_level = experience_this_level as f32 / 1000.0;
        let treasure_over_level = expected_combined_total_treasure_value_end_of_level - expected_combined_total_treasure_value_start_of_level;
        let expected_combined_total_treasure_value = treasure_over_level * fraction_through_level + expected_combined_total_treasure_value_start_of_level;
        let expected_combined_total_treasure_value = expected_combined_total_treasure_value.round();
        CampaignStats {
            level: r.level as u32,
            total_xp: r.total_experience.unwrap_or(0) as u32,
            experience_this_level: r.experience_this_level.unwrap_or(0) as u32,
            num_accomplishments: r.num_accomplishments.unwrap_or(0) as u32,
            num_combat_encounters: r.num_combat_encounters.unwrap_or(0) as u32,
            num_subsystem_encounters: r.num_subsystem_encounters.unwrap_or(0) as u32,
            num_sessions: r.num_sessions.unwrap_or(0) as u32,
            total_combined_treasure: r.total_combined_treasure_value.unwrap_or(0.0) as u32,
            total_expected_combined_treasure: expected_combined_total_treasure_value,
            total_treasure_items_value: r.total_treasure_items_value.unwrap_or(0.0) as u32,
            total_gold: r.total_treasure_currency_value.unwrap_or(0.0) as u32,
            total_expected_combined_treasure_start_of_level: r.expected_combined_total_treasure_value_start_of_level.unwrap_or(0.0) as f32,
            total_expected_combined_treasure_end_of_level: r.expected_combined_total_treasure_value_end_of_level.unwrap_or(0.0) as f32,
            encounters,
            total_permanent_items_by_level: permanent_items_by_level,
            expected_permanent_items_by_end_of_level,
            total_consumable_items_by_level: consumable_items_by_level,
            expected_consumable_items_by_end_of_level,
            character_stats: characters_query,
        }
    }).ok_or_else(|| crate::ServerError::NotFound)?;

    Ok(query)
}
