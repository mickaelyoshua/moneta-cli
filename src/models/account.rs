use super::types::{AccountType, NonEmptyString};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
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

#[derive(Debug)]
pub struct NewAccount {
    pub name: NonEmptyString,
    pub account_type: AccountType,
    pub has_debit_card: bool,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct AccountBalance {
    pub account_id: i32,
    pub balance: Decimal,
}

impl Account {
    pub async fn insert(pool: &sqlx::PgPool, new_acc: NewAccount) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO accounts (name, account_type, has_debit_card, active)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(new_acc.name.as_str())
        .bind(new_acc.account_type)
        .bind(new_acc.has_debit_card)
        .bind(new_acc.active)
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
            SELECT * FROM accounts
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn balance(
        pool: &sqlx::PgPool,
        account_id: i32,
    ) -> Result<AccountBalance, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(
                CASE 
                    WHEN transaction_type = 'income' THEN amount
                    ELSE -amount
                END
            ), 0) AS balance
            FROM transactions
            WHERE account_id = $1 AND status = 'cleared'
            "#,
            account_id
        )
        .fetch_one(pool)
        .await?;

        Ok(AccountBalance {
            account_id,
            balance: row.balance.unwrap_or(Decimal::ZERO),
        })
    }
}
