use serde::Serialize;
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use super::types::NonEmptyString;

#[derive(Debug, Serialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: NonEmptyString,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TransactionTag {
    pub transaction_id: i32,
    pub tag_id: i32,
}
