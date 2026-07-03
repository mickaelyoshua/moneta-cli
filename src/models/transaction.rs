use super::types::{
    NonEmptyString, PositiveAmount, TransactionSource, TransactionStatus, TransactionType,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub id: i32,
    pub category_id: i32,

    #[serde(flatten)]
    pub source: TransactionSource,

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

#[derive(Debug)]
pub struct NewTransaction {
    pub category_id: i32,
    pub source: TransactionSource,
    pub transaction_type: TransactionType,
    pub amount: PositiveAmount,
    pub date: NaiveDate,
    pub description: NonEmptyString,
}

impl Transaction {
    pub async fn insert(pool: &sqlx::PgPool, new_tx: NewTransaction) -> Result<Self, sqlx::Error> {
        let (account_id, credit_card_id) = match new_tx.source {
            TransactionSource::Account { account_id } => (Some(account_id), None),
            TransactionSource::CreditCard { credit_card_id } => (None, Some(credit_card_id)),
        };

        let row = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions (
                category_id, account_id, credit_card_id, 
                transaction_type, amount, date, description
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(new_tx.category_id)
        .bind(account_id)
        .bind(credit_card_id)
        .bind(new_tx.transaction_type)
        .bind(new_tx.amount.as_decimal())
        .bind(new_tx.date)
        .bind(new_tx.description.as_str())
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn find_all(
        pool: &sqlx::PgPool,
        limit: Option<usize>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let limit = limit.unwrap_or(50) as i64;
        let rows = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT *
            FROM transactions
            ORDER BY date DESC, created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for Transaction {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let account_id: Option<i32> = row.try_get("account_id")?;
        let credit_card_id: Option<i32> = row.try_get("credit_card_id")?;

        let source = match (account_id, credit_card_id) {
            (Some(id), None) => TransactionSource::Account { account_id: id },
            (None, Some(id)) => TransactionSource::CreditCard { credit_card_id: id },
            _ => {
                return Err(sqlx::Error::Decode(
                    "Transação deve pertencer a uma conta ou a um cartão de crédito.".into(),
                ));
            }
        };

        Ok(Self {
            id: row.try_get("id")?,
            category_id: row.try_get("category_id")?,
            source,
            installment_id: row.try_get("installment_id")?,
            recurrence_id: row.try_get("recurrence_id")?,
            transaction_type: row.try_get("transaction_type")?,
            amount: row.try_get("amount")?,
            date: row.try_get("date")?,
            description: row.try_get("description")?,
            installment_number: row.try_get("installment_number")?,
            status: row.try_get("status")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
