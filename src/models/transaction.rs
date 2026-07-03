use serde::Serialize;
use sqlx::FromRow;
use chrono::{DateTime, Utc, NaiveDate};
use super::types::{PositiveAmount, TransactionType, TransactionStatus, NonEmptyString};

#[derive(Debug, Serialize, FromRow)]
pub struct Transaction {
    pub id: i32,
    pub category_id: i32,
    pub account_id: Option<i32>,
    pub credit_card_id: Option<i32>,
    pub installment_id: Option<i32>,
    pub recurrence_id: Option<i32>,
    
    pub transaction_type: TransactionType,
    pub amount: PositiveAmount,
    pub date: NaiveDate,
    pub description: NonEmptyString,
    pub installment_number: Option<i16>,
    pub status: TransactionStatus,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
