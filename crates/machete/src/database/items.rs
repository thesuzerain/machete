use std::collections::HashMap;

use machete_core::filters::Filter;
use sqlx::{QueryBuilder, Row, Sqlite};

use crate::{
    models::library::{
        item::{Currency, ItemFilters, LibraryItem},
        GameSystem, Rarity,
    },
    MacheteError,
};

use super::QueryableStruct;

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_items(
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    condition: &ItemFilters,
) -> crate::Result<Vec<LibraryItem>> {
    // TODO: This doesn't use sqlx::query! because it needs to be dynamic. Is there a better way to do this?
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        r#"
        SELECT 
            lo.id,
            lo.name,
            lo.game_system,
            rarity,
            level,
            price,
            GROUP_CONCAT(tag) AS tags
        FROM library_objects lo
        INNER JOIN library_items li ON lo.id = li.id
        LEFT JOIN library_objects_tags lot ON lo.id = lot.library_object_id
        LEFT JOIN tags t ON lot.tag_id = t.id
    "#,
    );

    builder.push("WHERE 1=1 ");
    let mut builder = condition.build_filters(builder);
    builder.push(" GROUP BY lo.id ORDER BY lo.name"); // TODO: custom order

    let built = builder.build();
    let items = built
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(LibraryItem {
                name: row.get(1),
                game_system: GameSystem::from_i64(row.get(2)),
                rarity: Rarity::from_i64(row.get(3)),
                level: row.get(4),
                price: Currency::from_base_unit(row.get(5)),
                tags: row
                    .get::<String, _>(6)
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
            })
        })
        .collect::<Result<Vec<LibraryItem>, sqlx::Error>>()?;
    Ok(items)
}

pub async fn insert_items(
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    items: Vec<LibraryItem>,
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
    for (i, item) in items.iter().enumerate() {
        builder.push("(");
        builder.push_bind(&item.name);
        builder.push(", ");
        builder.push_bind(item.game_system.as_i64());
        builder.push(")");
        if i < items.len() - 1 {
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
        INSERT INTO library_items (id, rarity, level, price)
        VALUES
    "#,
    );

    for (i, (id, item)) in ids.iter().zip(items.iter()).enumerate() {
        builder.push("(");
        builder.push_bind(id);
        builder.push(", ");
        builder.push_bind(item.rarity.as_i64());
        builder.push(", ");
        builder.push_bind(item.level);
        builder.push(", ");
        builder.push_bind(item.price.as_base_unit());
        builder.push(")");
        if i < items.len() - 1 {
            builder.push(", ");
        }
    }

    let built = builder.build();
    built.execute(exec).await?;

    // Next, insert tags

    for (id, item) in ids.iter().zip(items.iter()) {
        // separate builders to not hit limit
        // todo: no :()
        let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
            r#"
            INSERT INTO library_objects_tags (library_object_id, tag_id)
            VALUES
        "#,
        );

        for (j, tag) in item.tags.iter().enumerate() {
            if let Some(tag_id) = tag_hashmap.get(tag) {
                builder.push("(");
                builder.push_bind(id);
                builder.push(", ");
                builder.push_bind(tag_id);
                builder.push(")");
                if j < item.tags.len() - 1 {
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

impl ItemFilters {
    // TODO: is this redundant with the other conversion? can do one conversion?
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
        if let Some(ref min_price) = self.min_price {
            query_builder.push(" AND price >= ");
            query_builder.push_bind(min_price);
        }
        if let Some(ref max_price) = self.max_price {
            query_builder.push(" AND price <= ");
            query_builder.push_bind(max_price);
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

impl QueryableStruct for LibraryItem {
    // TODO: bundle with macro?
    async fn query_get<'a>(
        exec: impl sqlx::Executor<'a, Database = sqlx::Sqlite>,
        filters: &Vec<Filter<LibraryItem>>,
    ) -> crate::Result<Vec<LibraryItem>> {
        let mut item_filters = ItemFilters::default();
        for filter in filters {
            // TODO: clone
            let filter = filter.clone();
            if let Ok(filter) = ItemFilters::try_from(filter) {
                item_filters = item_filters.merge(filter);
            }
        }

        get_items(exec, &item_filters).await
    }
}
