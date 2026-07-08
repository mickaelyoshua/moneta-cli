use super::types::InvoiceStatus;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Invoice {
    pub id: i32,
    pub credit_card_id: i32,
    pub month: i16,
    pub year: i16,
    pub status: InvoiceStatus,
    pub total_amount: Decimal,
    pub due_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Invoice {
    pub async fn find_all_by_card(
        pool: &sqlx::PgPool,
        credit_card_id: i32,
        limit: Option<usize>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let limit = limit.unwrap_or(100) as i64;
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM invoices
            WHERE credit_card_id = $1
            ORDER BY year DESC, month DESC
            LIMIT $2
            "#,
        )
        .bind(credit_card_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn find_or_create_for_date(
        pool: &sqlx::PgPool,
        credit_card_id: i32,
        transaction_date: NaiveDate,
    ) -> Result<Self, sqlx::Error> {
        use chrono::Datelike;

        let card = sqlx::query!(
            r#"SELECT billing_day, due_day FROM credit_cards WHERE id = $1"#,
            credit_card_id
        )
        .fetch_one(pool)
        .await?;

        let day = transaction_date.day() as i16;
        let mut invoice_date = transaction_date;
        if day > card.billing_day {
            invoice_date = invoice_date + chrono::Months::new(1);
        }

        let month = invoice_date.month() as i16;
        let year = invoice_date.year() as i16;

        let existing = sqlx::query_as::<_, Self>(
            r#"SELECT * FROM invoices WHERE credit_card_id = $1 AND month = $2 AND year = $3"#,
        )
        .bind(credit_card_id)
        .bind(month)
        .bind(year)
        .fetch_optional(pool)
        .await?;

        if let Some(inv) = existing {
            return Ok(inv);
        }

        let mut due_month = month as u32;
        let mut due_year = year as i32;
        if card.due_day < card.billing_day {
            let next =
                NaiveDate::from_ymd_opt(due_year, due_month, 1).unwrap() + chrono::Months::new(1);
            due_month = next.month();
            due_year = next.year();
        }
        let due_date = NaiveDate::from_ymd_opt(due_year, due_month, card.due_day as u32).unwrap();

        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO invoices (credit_card_id, month, year, status, total_amount, due_date)
            VALUES ($1, $2, $3, 'open', 0, $4)
            RETURNING *
            "#,
        )
        .bind(credit_card_id)
        .bind(month)
        .bind(year)
        .bind(due_date)
        .fetch_one(pool)
        .await
    }

    pub async fn close(
        pool: &sqlx::PgPool,
        credit_card_id: i32,
        month: i16,
        year: i16,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE invoices 
            SET status = 'closed', updated_at = NOW() 
            WHERE credit_card_id = $1 AND month = $2 AND year = $3 AND status = 'open'
            RETURNING *
            "#,
        )
        .bind(credit_card_id)
        .bind(month)
        .bind(year)
        .fetch_one(pool)
        .await
    }

    pub async fn pay(
        pool: &sqlx::PgPool,
        credit_card_id: i32,
        month: i16,
        year: i16,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE invoices 
            SET status = 'paid', updated_at = NOW() 
            WHERE credit_card_id = $1 AND month = $2 AND year = $3 AND status = 'closed'
            RETURNING *
            "#,
        )
        .bind(credit_card_id)
        .bind(month)
        .bind(year)
        .fetch_one(pool)
        .await
    }

    pub async fn reopen(
        pool: &sqlx::PgPool,
        credit_card_id: i32,
        month: i16,
        year: i16,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE invoices 
            SET status = 'open', updated_at = NOW() 
            WHERE credit_card_id = $1 AND month = $2 AND year = $3 AND status = 'closed'
            RETURNING *
            "#,
        )
        .bind(credit_card_id)
        .bind(month)
        .bind(year)
        .fetch_one(pool)
        .await
    }
}
