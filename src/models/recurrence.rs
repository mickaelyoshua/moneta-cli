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

#[derive(Debug)]
pub struct NewRecurrence {
    pub category_id: i32,
    pub source: TransactionSource,
    pub transaction_type: TransactionType,
    pub amount: PositiveAmount,
    pub description: NonEmptyString,
    pub frequency: RecurrenceFrequency,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug)]
pub struct UpdateRecurrencePayload {
    pub category_id: Option<i32>,
    pub account_id: Option<i32>,
    pub credit_card_id: Option<i32>,
    pub transaction_type: Option<TransactionType>,
    pub amount: Option<PositiveAmount>,
    pub description: Option<NonEmptyString>,
    pub frequency: Option<RecurrenceFrequency>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<Option<NaiveDate>>,
}

impl Recurrence {
    pub async fn insert(pool: &sqlx::PgPool, new: NewRecurrence) -> Result<Self, crate::models::ModelError> {
        let (account_id, credit_card_id) = match new.source {
            TransactionSource::Account { account_id } => (Some(account_id), None),
            TransactionSource::CreditCard { credit_card_id } => (None, Some(credit_card_id)),
        };

        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO recurrences 
            (category_id, account_id, credit_card_id, transaction_type, amount, description, frequency, start_date, end_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(new.category_id)
        .bind(account_id)
        .bind(credit_card_id)
        .bind(new.transaction_type)
        .bind(new.amount.as_decimal())
        .bind(new.description.as_str())
        .bind(new.frequency)
        .bind(new.start_date)
        .bind(new.end_date)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn find_all(pool: &sqlx::PgPool) -> Result<Vec<Self>, crate::models::ModelError> {
        sqlx::query_as::<_, Self>("SELECT * FROM recurrences ORDER BY start_date DESC")
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn delete(pool: &sqlx::PgPool, id: i32) -> Result<(), crate::models::ModelError> {
        sqlx::query!("DELETE FROM recurrences WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(
        pool: &sqlx::PgPool,
        id: i32,
        payload: UpdateRecurrencePayload,
    ) -> Result<Self, crate::models::ModelError> {
        let mut db_tx = pool.begin().await?;

        let old = sqlx::query_as::<_, Self>("SELECT * FROM recurrences WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *db_tx)
            .await?;

        let old = match old {
            Some(t) => t,
            None => return Err(crate::models::ModelError::NotFound),
        };

        let mut next_account_id = match old.source {
            TransactionSource::Account { account_id } => Some(account_id),
            _ => None,
        };
        let mut next_credit_card_id = match old.source {
            TransactionSource::CreditCard { credit_card_id } => Some(credit_card_id),
            _ => None,
        };

        if let Some(acc_id) = payload.account_id {
            next_account_id = Some(acc_id);
            next_credit_card_id = None;
        } else if let Some(cc_id) = payload.credit_card_id {
            next_credit_card_id = Some(cc_id);
            next_account_id = None;
        }

        let row = sqlx::query_as::<_, Self>(
            r#"
            UPDATE recurrences
            SET 
                category_id = COALESCE($1, category_id),
                account_id = $2,
                credit_card_id = $3,
                transaction_type = COALESCE($4, transaction_type),
                amount = COALESCE($5, amount),
                description = COALESCE($6, description),
                frequency = COALESCE($7, frequency),
                start_date = COALESCE($8, start_date),
                end_date = COALESCE($9, end_date)
            WHERE id = $10
            RETURNING *
            "#,
        )
        .bind(payload.category_id)
        .bind(next_account_id)
        .bind(next_credit_card_id)
        .bind(payload.transaction_type)
        .bind(payload.amount.map(|a| a.as_decimal()))
        .bind(payload.description.as_ref().map(|d| d.as_str()))
        .bind(payload.frequency)
        .bind(payload.start_date)
        .bind(payload.end_date.unwrap_or(old.end_date))
        .bind(id)
        .fetch_one(&mut *db_tx)
        .await?;

        db_tx.commit().await?;
        Ok(row)
    }

    pub async fn sync_all(pool: &sqlx::PgPool, ref_date: NaiveDate) -> Result<usize, crate::models::ModelError> {
        let mut db_tx = pool.begin().await?;

        let recurrences = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM recurrences 
            WHERE 
                (end_date IS NULL OR start_date <= end_date) AND
                (last_processed_date IS NULL OR last_processed_date < $1)
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(ref_date)
        .fetch_all(&mut *db_tx)
        .await?;

        let mut inserted_count = 0;

        for mut rec in recurrences {
            let mut current = if let Some(last) = rec.last_processed_date {
                rec.next_date(last)
            } else {
                rec.start_date
            };

            while current <= ref_date {
                if let Some(end) = rec.end_date
                    && current > end
                {
                    break;
                }

                let new_tx = crate::models::transaction::NewTransaction {
                    category_id: Some(rec.category_id),
                    source: rec.source.clone(),
                    transaction_type: rec.transaction_type,
                    amount: rec.amount,
                    date: current,
                    description: rec.description.clone(),
                    installment_id: None,
                    installment_number: None,
                    tags: vec![],
                };

                let tx_res =
                    crate::models::transaction::Transaction::insert(&mut db_tx, new_tx).await;
                match tx_res {
                    Ok(tx) => {
                        sqlx::query!(
                            "UPDATE transactions SET recurrence_id = $1 WHERE id = $2",
                            rec.id,
                            tx.id
                        )
                        .execute(&mut *db_tx)
                        .await?;

                        inserted_count += 1;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }

                rec.last_processed_date = Some(current);
                current = rec.next_date(current);
            }

            if rec.last_processed_date.is_some() {
                sqlx::query!(
                    "UPDATE recurrences SET last_processed_date = $1 WHERE id = $2",
                    rec.last_processed_date,
                    rec.id
                )
                .execute(&mut *db_tx)
                .await?;
            }
        }

        db_tx.commit().await?;
        Ok(inserted_count)
    }

    fn next_date(&self, from: NaiveDate) -> NaiveDate {
        use chrono::Datelike;
        match self.frequency {
            RecurrenceFrequency::Daily => from + chrono::Days::new(1),
            RecurrenceFrequency::Weekly => from + chrono::Days::new(7),
            RecurrenceFrequency::Monthly => {
                let mut m = from.month() + 1;
                let mut y = from.year();
                if m > 12 {
                    m = 1;
                    y += 1;
                }
                crate::models::safe_from_ymd(y, m, self.start_date.day())
            }
            RecurrenceFrequency::Yearly => {
                let y = from.year() + 1;
                crate::models::safe_from_ymd(y, from.month(), self.start_date.day())
            }
        }
    }
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
                    "Recurrence must belong to account or CC.".into(),
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
