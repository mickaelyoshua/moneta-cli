use super::types::{NonEmptyString, NonNegativeAmount};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct CreditCard {
    pub id: i32,
    pub account_id: i32,
    pub name: NonEmptyString,
    pub credit_limit: NonNegativeAmount,
    pub billing_day: i16,
    pub due_day: i16,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
