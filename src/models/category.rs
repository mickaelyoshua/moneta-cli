use super::types::{CategoryType, NonEmptyString};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Category {
    pub id: i32,
    pub name: NonEmptyString,
    pub category_type: CategoryType,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewCategory {
    pub name: NonEmptyString,
    pub category_type: CategoryType,
    pub active: bool,
}

impl Category {
    pub async fn insert(pool: &sqlx::PgPool, new_ctg: NewCategory) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            INSERT INTO categories (name, category_type, active)
            VALUES ($1, $2, $3)
            RETURNING id, name as "name: _", category_type as "category_type: _", active, created_at, updated_at
            "#,
            new_ctg.name.as_str(),
            new_ctg.category_type as _,
            new_ctg.active,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn find_all(
        pool: &sqlx::PgPool,
        limit: Option<usize>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let limit = limit.unwrap_or(100) as i64;

        sqlx::query_as!(
            Self,
            r#"
            SELECT id, name as "name: _", category_type as "category_type: _", active, created_at, updated_at
            FROM categories
            ORDER BY created_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(pool)
        .await
    }
}
