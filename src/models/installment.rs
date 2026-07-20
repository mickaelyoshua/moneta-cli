use super::types::{InstallmentCount, InstallmentNumber, NonEmptyString, PositiveAmount};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;
use std::str::FromStr;

#[derive(Debug, Serialize, FromRow)]
pub struct Installment {
    pub id: i32,
    pub credit_card_id: i32,
    pub description: NonEmptyString,
    pub total_amount: PositiveAmount,
    pub installments_count: InstallmentCount,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct InstallmentDetails {
    pub installment: Installment,
    pub transactions: Vec<crate::models::transaction::Transaction>,
}

#[derive(Debug)]
pub struct NewInstallment {
    pub credit_card_id: i32,
    pub category_id: Option<i32>,
    pub description: NonEmptyString,
    pub total_amount: PositiveAmount,
    pub installments_count: InstallmentCount,
    pub date: NaiveDate,
}

impl Installment {
    pub async fn insert(
        pool: &sqlx::PgPool,
        new_inst: NewInstallment,
    ) -> Result<Self, crate::models::ModelError> {
        let mut tx = pool.begin().await?;

        // 1. Insert base installment
        let installment = sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO installments (credit_card_id, description, total_amount, installments_count)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(new_inst.credit_card_id)
        .bind(new_inst.description.as_str())
        .bind(new_inst.total_amount.as_decimal())
        .bind(new_inst.installments_count.get())
        .fetch_one(&mut *tx)
        .await?;

        // 2. Apportionment logic (remainder in last installment)
        let total = new_inst.total_amount.as_decimal();
        let count_dec = Decimal::from(new_inst.installments_count.get());

        let base_amount = (total / count_dec).trunc_with_scale(2);

        // Check if base is 0
        if base_amount == Decimal::ZERO {
            return Err(crate::models::ModelError::BusinessLogic(
                "Installment amount too small for this count (min 0.01/installment).".into(),
            ));
        }

        let total_base = base_amount * Decimal::from(new_inst.installments_count.get() - 1);
        let last_amount = total - total_base;

        // 3. Generate monthly transactions
        for i in 1..=new_inst.installments_count.get() {
            let tx_date = new_inst.date + chrono::Months::new((i - 1) as u32);
            let tx_amount = if i == new_inst.installments_count.get() {
                PositiveAmount::try_from(last_amount)
                    .map_err(|e| crate::models::ModelError::BusinessLogic(e.into()))?
            } else {
                PositiveAmount::try_from(base_amount)
                    .map_err(|e| crate::models::ModelError::BusinessLogic(e.into()))?
            };

            let tx_desc = NonEmptyString::from_str(&format!(
                "{} ({}/{})",
                new_inst.description.as_str(),
                i,
                new_inst.installments_count.get()
            ))
            .map_err(|e| crate::models::ModelError::BusinessLogic(e.into()))?;

            let new_tx = crate::models::transaction::NewTransaction {
                category_id: new_inst.category_id,
                source: crate::models::types::TransactionSource::CreditCard {
                    credit_card_id: new_inst.credit_card_id,
                },
                transaction_type: crate::models::types::TransactionType::Expense, // CC always generates Expense
                amount: tx_amount,
                date: tx_date,
                description: tx_desc,
                installment_id: Some(installment.id),
                installment_number: Some(
                    InstallmentNumber::try_from(i)
                        .map_err(|e| crate::models::ModelError::BusinessLogic(e.into()))?,
                ),
                tags: vec![],
            };

            // Reuse tx logic
            crate::models::transaction::Transaction::insert(&mut tx, new_tx).await?;
        }

        tx.commit().await?;
        Ok(installment)
    }

    pub async fn find_all(
        pool: &sqlx::PgPool,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Self>, crate::models::ModelError> {
        let limit = limit.unwrap_or(100) as i64;
        let offset = offset.unwrap_or(0) as i64;
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM installments
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn find_by_id(
        pool: &sqlx::PgPool,
        id: i32,
    ) -> Result<Self, crate::models::ModelError> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM installments
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn delete(pool: &sqlx::PgPool, id: i32) -> Result<bool, crate::models::ModelError> {
        let mut tx = pool.begin().await?;

        // Check if linked to PAID invoice
        let paid_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM transactions t
            JOIN invoices i ON t.invoice_id = i.id
            WHERE t.installment_id = $1 AND i.status != 'open'
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        if paid_count > 0 {
            return Err(crate::models::ModelError::BusinessLogic(
                "Budget/Invoice corresponding to one of the installments is already closed/paid.".into(),
            ));
        }

        // Invoices are dynamic. Just delete transactions.

        sqlx::query!(r#"DELETE FROM transactions WHERE installment_id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query!(r#"DELETE FROM installments WHERE id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn adjust(
        pool: &sqlx::PgPool,
        installment_id: i32,
        number: InstallmentNumber,
        new_amount: PositiveAmount,
    ) -> Result<crate::models::transaction::Transaction, crate::models::ModelError> {
        let mut tx = pool.begin().await?;

        // Find transaction
        let old_tx = sqlx::query_as::<_, crate::models::transaction::Transaction>(
            r#"SELECT * FROM transactions WHERE installment_id = $1 AND installment_number = $2"#,
        )
        .bind(installment_id)
        .bind(number.get())
        .fetch_optional(&mut *tx)
        .await?;

        let old_tx = match old_tx {
            Some(t) => t,
            None => return Err(crate::models::ModelError::NotFound),
        };

        if let Some(inv_id) = old_tx.invoice_id {
            // Check if invoice not closed/paid
            let status = sqlx::query_scalar::<_, crate::models::types::InvoiceStatus>(
                "SELECT status FROM invoices WHERE id = $1",
            )
            .bind(inv_id)
            .fetch_one(&mut *tx)
            .await?;

            if status != crate::models::types::InvoiceStatus::Open {
                return Err(crate::models::ModelError::BusinessLogic(
                    "Cannot change an installment of a closed invoice/budget.".into(),
                ));
            }
            // Invoice total is calculated on-the-fly.
        }

        let updated_tx = sqlx::query_as::<_, crate::models::transaction::Transaction>(
            r#"
            UPDATE transactions
            SET amount = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(new_amount.as_decimal())
        .bind(old_tx.id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(updated_tx)
    }

    pub async fn find_or_create_for_import(
        pool: &sqlx::PgPool,
        credit_card_id: i32,
        category_id: Option<i32>,
        description: NonEmptyString,
        total_amount: PositiveAmount,
        installments_count: InstallmentCount,
        original_date: NaiveDate,
    ) -> Result<Self, crate::models::ModelError> {
        let exists = sqlx::query_as::<_, Self>(
            r#"SELECT * FROM installments WHERE credit_card_id = $1 AND description = $2 AND total_amount = $3"#
        )
        .bind(credit_card_id)
        .bind(description.as_str())
        .bind(total_amount.as_decimal())
        .fetch_optional(pool)
        .await?;

        if let Some(inst) = exists {
            return Ok(inst);
        }

        let new_inst = NewInstallment {
            credit_card_id,
            category_id,
            description,
            total_amount,
            installments_count,
            date: original_date,
        };
        Self::insert(pool, new_inst).await
    }
}
