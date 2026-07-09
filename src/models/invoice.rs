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
    pub closing_amount: Option<Decimal>,
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
        conn: &mut sqlx::PgConnection,
        credit_card_id: i32,
        transaction_date: NaiveDate,
    ) -> Result<Self, sqlx::Error> {
        use chrono::Datelike;

        let card = sqlx::query!(
            r#"SELECT billing_day, due_day FROM credit_cards WHERE id = $1"#,
            credit_card_id
        )
        .fetch_one(&mut *conn)
        .await?;

        let day = transaction_date.day() as i16;
        let mut invoice_date = transaction_date;
        if day >= card.billing_day {
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
        .fetch_optional(&mut *conn)
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
            INSERT INTO invoices (credit_card_id, month, year, status, due_date)
            VALUES ($1, $2, $3, 'open', $4)
            RETURNING *
            "#,
        )
        .bind(credit_card_id)
        .bind(month)
        .bind(year)
        .bind(due_date)
        .fetch_one(&mut *conn)
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
            SET status = 'closed', closing_amount = COALESCE((
                SELECT SUM(
                    CASE 
                        WHEN transaction_type = 'income' THEN -amount 
                        ELSE amount 
                    END
                ) FROM transactions WHERE invoice_id = invoices.id
            ), 0)
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
        account_id: i32,
    ) -> Result<Self, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let invoice = sqlx::query_as::<_, Self>(
            r#"
            UPDATE invoices 
            SET status = 'paid'
            WHERE credit_card_id = $1 AND month = $2 AND year = $3 AND status = 'closed'
            RETURNING *
            "#,
        )
        .bind(credit_card_id)
        .bind(month)
        .bind(year)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO transactions (
                account_id, transaction_type, amount, date, description, status
            )
            VALUES ($1, 'transfer'::transaction_type_enum, $2, CURRENT_DATE, $3, 'cleared'::transaction_status_enum)
            "#,
            account_id,
            invoice.closing_amount.unwrap_or(Decimal::ZERO),
            format!("Pagamento Fatura {}/{}", month, year)
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(invoice)
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
            SET status = 'open', closing_amount = NULL
            WHERE credit_card_id = $1 AND month = $2 AND year = $3 AND status = 'closed'
            RETURNING *
            "#
        )
        .bind(credit_card_id)
        .bind(month)
        .bind(year)
        .fetch_one(pool)
        .await
    }

    pub async fn current_total(
        pool: &sqlx::PgPool,
        invoice_id: i32,
    ) -> Result<Decimal, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(
                CASE 
                    WHEN transaction_type = 'income' THEN -amount 
                    ELSE amount 
                END
            ), 0) AS total
            FROM transactions
            WHERE invoice_id = $1
            "#,
            invoice_id
        )
        .fetch_one(pool)
        .await?;

        Ok(row.total.unwrap_or(Decimal::ZERO))
    }
}
