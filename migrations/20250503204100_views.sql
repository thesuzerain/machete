CREATE OR REPLACE VIEW campaign_sessions_enhanced AS
SELECT
    o.id,
    o.campaign_id,
    any_value(o.session_order) as session_order,
    any_value(session_experience) as session_experience,
    any_value(cumulative_experience) as cumulative_experience,
    any_value(current_level) as current_level,
    any_value(cumulative_treasure_currency) as cumulative_treasure_currency,
    any_value(cumulative_treasure_items) as cumulative_treasure_items,
    any_value((ex.total_value * session_experience/1000)) as expected_total_treasure
FROM (
    SELECT
        cs.id,
        cs.campaign_id,
        cs.session_order,
        SUM(e.total_experience) as session_experience,
        SUM(e.treasure_currency) as session_treasure_currency,
        SUM(e.total_items_value) as session_items_value,

        SUM(SUM(e.total_experience)) OVER (PARTITION BY campaign_id ORDER BY session_order ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) cumulative_experience,
        1+(SUM(SUM(e.total_experience)) OVER (PARTITION BY campaign_id ORDER BY session_order ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW)/1000) current_level,
        SUM(SUM(e.treasure_currency)) OVER (PARTITION BY campaign_id ORDER BY session_order ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) cumulative_treasure_currency,
        SUM(SUM(e.total_items_value) ) OVER (PARTITION BY campaign_id ORDER BY session_order ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) cumulative_treasure_items

    FROM campaign_sessions cs
    INNER JOIN encounters e ON e.session_id = cs.id
    GROUP BY cs.id
) o
INNER JOIN expected_treasures_by_level ex ON ex.level = FLOOR(o.current_level)
GROUP BY o.id, campaign_id;
