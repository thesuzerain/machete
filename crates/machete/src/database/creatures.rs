use std::collections::HashMap;

use sqlx::{QueryBuilder, Row, Sqlite};

use crate::{
    filters::filter::Filter,
    models::library::{
        creature::{Alignment, CreatureFilters, LibraryCreature, Size},
        GameSystem, Rarity,
    },
    MacheteError,
};

use super::QueryableStruct;

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_creatures(
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    condition: &CreatureFilters,
) -> crate::Result<Vec<LibraryCreature>> {
    // TODO: This doesn't use sqlx::query! because it needs to be dynamic. Is there a better way to do this?
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        r#"
        SELECT 
            lo.id,
            lo.name,
            lo.game_system,
            rarity,
            level,
            alignment,
            size,
            GROUP_CONCAT(tag) AS tags
        FROM library_objects lo
        INNER JOIN library_creatures lc ON lo.id = lc.id
        LEFT JOIN library_objects_tags lot ON lo.id = lot.library_object_id
        LEFT JOIN tags t ON lot.tag_id = t.id
    "#,
    );

    builder.push("WHERE 1=1 ");
    let mut builder = condition.build_filters(builder);
    builder.push(" GROUP BY lo.id ORDER BY lo.name"); // TODO: custom order

    let built = builder.build();
    let creatures = built
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(LibraryCreature {
                name: row.get(1),
                game_system: GameSystem::from_i64(row.get(2)),
                rarity: Rarity::from_i64(row.get(3)),
                level: row.get(4),
                alignment: Alignment::from_i64(row.get(5)),
                size: Size::from_i64(row.get(6)),
                tags: row
                    .get::<String, _>(7)
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
            })
        })
        .collect::<Result<Vec<LibraryCreature>, sqlx::Error>>()?;
    Ok(creatures)
}

pub async fn insert_creatures(
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    creatures: Vec<LibraryCreature>,
    // not sure if i like this patern, but if we are keeping it, document it
    tag_hashmap: HashMap<String, i64>,
) -> crate::Result<()> {
    // TODO: This doesn't use sqlx::query! because it needs to be dynamic. Is there a better way to do this?
    // Maybe postgres + unnest as in labrinth?
    // TODO: Do we *need* two tables for this?
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        r#"
        INSERT INTO library_objects (name, game_system)
        VALUES    
    "#,
    );

    // TODO: I really don't like this
    for (i, creature) in creatures.iter().enumerate() {
        builder.push("(");
        builder.push_bind(&creature.name);
        builder.push(", ");
        builder.push_bind(creature.game_system.as_i64());
        builder.push(")");
        if i < creatures.len() - 1 {
            builder.push(", ");
        }
    }
    builder.push(
        "
        RETURNING id
    ",
    );

    let built = builder.build();
    let ids = built
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| Ok(row.get(0)))
        .collect::<Result<Vec<i64>, sqlx::Error>>()?;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        r#"
        INSERT INTO library_creatures (id, rarity, level, alignment, size)
        VALUES
    "#,
    );

    for (i, (id, creature)) in ids.iter().zip(creatures.iter()).enumerate() {
        builder.push("(");
        builder.push_bind(id);
        builder.push(", ");
        builder.push_bind(creature.rarity.as_i64());
        builder.push(", ");
        builder.push_bind(creature.level);
        builder.push(", ");
        builder.push_bind(creature.alignment.as_i64());
        builder.push(", ");
        builder.push_bind(creature.size.as_i64());
        builder.push(")");
        if i < creatures.len() - 1 {
            builder.push(", ");
        }
    }

    let built = builder.build();
    built.execute(exec).await?;

    // Next, insert tags
    for (id, creature) in ids.iter().zip(creatures.iter()) {
        // separate builders to not hit limit
        // todo: no :()
        let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
            r#"
            INSERT INTO library_objects_tags (library_object_id, tag_id)
            VALUES
        "#,
        );

        for (j, tag) in creature.tags.iter().enumerate() {
            if let Some(tag_id) = tag_hashmap.get(tag) {
                builder.push("(");
                builder.push_bind(id);
                builder.push(", ");
                builder.push_bind(tag_id);
                builder.push(")");
                if j < creature.tags.len() - 1 {
                    builder.push(", ");
                }
            } else {
                return Err(MacheteError::InternalError("Tag not found".to_string()));
            }
        }

        let built = builder.build();

        built.execute(exec).await?;
    }

    Ok(())
}

impl CreatureFilters {
    // TODO: is this redundant with the other conversion? can do one conversion?
    // TODO: multiple impls of creaturefilters
    pub fn build_filters<'a>(
        &'a self,
        mut query_builder: QueryBuilder<'a, Sqlite>,
    ) -> QueryBuilder<'a, Sqlite> {
        if let Some(min_level) = self.min_level {
            query_builder.push(" AND level >= ");
            query_builder.push_bind(min_level);
        }
        if let Some(max_level) = self.max_level {
            query_builder.push(" AND level <= ");
            query_builder.push_bind(max_level);
        }
        if let Some(ref alignment) = self.alignment {
            query_builder.push(" AND alignment = ");
            query_builder.push_bind(alignment.as_i64());
        }
        if let Some(ref size) = self.size {
            query_builder.push(" AND size = ");
            query_builder.push_bind(size.as_i64());
        }
        if let Some(ref name) = self.name {
            // TODO: when autobuilding th-s all should hagve db specific?
            query_builder.push(" AND lo.name LIKE '%' || ");
            query_builder.push_bind(name);
            query_builder.push(" || '%'");
        }
        if let Some(ref rarity) = self.rarity {
            query_builder.push(" AND rarity = ");
            query_builder.push_bind(rarity.as_i64());
        }
        if let Some(ref game_system) = self.game_system {
            query_builder.push(" AND game_system = ");
            query_builder.push_bind(game_system.as_i64());
        }
        if !self.tags.is_empty() {
            // TODO: I think maybe all the string fields should function like this so we can do 'or' filters OK
            query_builder.push(" AND (");
            for (i, tag) in self.tags.iter().enumerate() {
                query_builder.push("tag LIKE '%' || ");
                query_builder.push_bind(tag);
                query_builder.push(" || '%'");
                if i < self.tags.len() - 1 {
                    query_builder.push(" OR ");
                }
            }
            query_builder.push(")");
        }

        query_builder
    }
}

impl QueryableStruct for LibraryCreature {
    async fn query_get<'a>(
        exec: impl sqlx::Executor<'a, Database = sqlx::Sqlite>,
        filters: &Vec<Filter<LibraryCreature>>,
    ) -> crate::Result<Vec<LibraryCreature>> {
        let mut creature_filters = CreatureFilters::default();
        for filter in filters {
            let filter = (*filter).clone();
            // TODO: include with macro...? or at least better functions?
            // todo: remove clone
            if let Some(cf) = filter.to_creature_filter() {
                creature_filters = creature_filters.merge(cf);
            }
        }

        get_creatures(exec, &creature_filters).await
    }
}
