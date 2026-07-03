use serde::Serialize;
use sqlx::FromRow;
use chrono::{DateTime, Utc, NaiveDate};
use super::types::{PositiveAmount, TransactionType, RecurrenceFrequency, NonEmptyString};

#[derive(Debug, Serialize, FromRow)]
pub struct Recurrence {
    pub id: i32,
    pub category_id: i32,
    pub account_id: Option<i32>,
    pub credit_card_id: Option<i32>,
    
    pub transaction_type: TransactionType,
    pub amount: PositiveAmount,
    pub description: NonEmptyString,
    pub frequency: RecurrenceFrequency,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub last_processed_date: Option<NaiveDate>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
