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

#[derive(Debug)]
pub struct NewCreditCard {
    pub account_id: i32,
    pub name: NonEmptyString,
    pub credit_limit: NonNegativeAmount,
    pub billing_day: super::types::DayOfMonth,
    pub due_day: super::types::DayOfMonth,
    pub active: bool,
}

impl CreditCard {
    pub async fn insert(pool: &sqlx::PgPool, new_card: NewCreditCard) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO credit_cards (account_id, name, credit_limit, billing_day, due_day, active)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(new_card.account_id)
        .bind(new_card.name.as_str())
        .bind(new_card.credit_limit)
        .bind(new_card.billing_day.as_i16())
        .bind(new_card.due_day.as_i16())
        .bind(new_card.active)
        .fetch_one(pool)
        .await
    }

    pub async fn find_all(
        pool: &sqlx::PgPool,
        limit: Option<usize>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let limit = limit.unwrap_or(100) as i64;
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM credit_cards
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}
