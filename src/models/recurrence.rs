use super::types::{
    NonEmptyString, PositiveAmount, RecurrenceFrequency, TransactionSource, TransactionType,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Recurrence {
    pub id: i32,
    pub category_id: i32,

    #[serde(flatten)]
    pub source: TransactionSource,

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

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for Recurrence {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let account_id: Option<i32> = row.try_get("account_id")?;
        let credit_card_id: Option<i32> = row.try_get("credit_card_id")?;

        let source = match (account_id, credit_card_id) {
            (Some(id), None) => TransactionSource::Account { account_id: id },
            (None, Some(id)) => TransactionSource::CreditCard { credit_card_id: id },
            _ => {
                return Err(sqlx::Error::Decode(
                    "Recorrência deve pertencer a uma conta ou a um cartão de crédito.".into(),
                ));
            }
        };

        Ok(Self {
            id: row.try_get("id")?,
            category_id: row.try_get("category_id")?,
            source,
            transaction_type: row.try_get("transaction_type")?,
            amount: row.try_get("amount")?,
            description: row.try_get("description")?,
            frequency: row.try_get("frequency")?,
            start_date: row.try_get("start_date")?,
            end_date: row.try_get("end_date")?,
            last_processed_date: row.try_get("last_processed_date")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
