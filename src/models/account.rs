use super::types::{AccountType, NonEmptyString};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: i32,
    pub name: NonEmptyString,
    pub account_type: AccountType,
    pub has_debit_card: bool,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
