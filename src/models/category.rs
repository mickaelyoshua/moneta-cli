use serde::Serialize;
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use super::types::{CategoryType, NonEmptyString};

#[derive(Debug, Serialize, FromRow)]
pub struct Category {
    pub id: i32,
    pub name: NonEmptyString,
    pub category_type: CategoryType,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
