use super::types::{NonEmptyString, PositiveAmount};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Installment {
    pub id: i32,
    pub credit_card_id: i32,
    pub description: NonEmptyString,
    pub total_amount: PositiveAmount,
    pub installments_count: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
