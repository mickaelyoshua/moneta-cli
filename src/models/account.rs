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
            SELECT COALESCE(SUM(vt.account_effect), 0) AS balance
            FROM transactions t
            INNER JOIN v_transaction_totals vt ON vt.transaction_id = t.id
            WHERE t.account_id = $1 AND t.status = 'cleared'
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

    pub async fn find_by_id(pool: &sqlx::PgPool, id: i32) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM accounts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn update(&self, pool: &sqlx::PgPool) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE accounts
            SET name = $1, account_type = $2, has_debit_card = $3, active = $4, updated_at = NOW()
            WHERE id = $5
            RETURNING *
            "#,
        )
        .bind(self.name.as_str())
        .bind(self.account_type)
        .bind(self.has_debit_card)
        .bind(self.active)
        .bind(self.id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &sqlx::PgPool, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM accounts
            WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_by_name(
        pool: &sqlx::PgPool,
        name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM accounts
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await
    }
}
