use super::types::{NonEmptyString, PositiveAmount};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::FromRow;
use std::str::FromStr;

#[derive(Debug, Serialize, FromRow)]
pub struct Installment {
    pub id: i32,
    pub credit_card_id: i32,
    pub description: NonEmptyString,
    pub total_amount: PositiveAmount,
    pub installments_count: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewInstallment {
    pub credit_card_id: i32,
    pub category_id: Option<i32>,
    pub description: NonEmptyString,
    pub total_amount: PositiveAmount,
    pub installments_count: i16,
    pub date: NaiveDate,
}

impl Installment {
    pub async fn insert(
        pool: &sqlx::PgPool,
        new_inst: NewInstallment,
    ) -> Result<Self, sqlx::Error> {
        let mut tx = pool.begin().await?;

        // 1. Inserir o Installment base
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
        .bind(new_inst.installments_count)
        .fetch_one(&mut *tx)
        .await?;

        // 2. Lógica de rateio com sobra para a última parcela
        let total = new_inst.total_amount.as_decimal();
        let count_dec = Decimal::from_str(&new_inst.installments_count.to_string()).unwrap();
        
        let base_amount = (total / count_dec).trunc_with_scale(2);

        // Verifica se a base ficou 0 (ex: total 0.05 em 10 vezes = 0.00 base, erro de regra de negócio)
        if base_amount == Decimal::ZERO {
            return Err(sqlx::Error::Protocol(
                "O valor do parcelamento é pequeno demais para essa quantidade de parcelas (mínimo R$ 0,01 por parcela).".into()
            ));
        }

        let total_base = base_amount
            * Decimal::from_str(&(new_inst.installments_count - 1).to_string()).unwrap();
        let last_amount = total - total_base;

        // 3. Gerar transações mensais
        for i in 1..=new_inst.installments_count {
            let tx_date = new_inst.date + chrono::Months::new((i - 1) as u32);
            let tx_amount = if i == new_inst.installments_count {
                PositiveAmount::from_str(&last_amount.to_string()).unwrap()
            } else {
                PositiveAmount::from_str(&base_amount.to_string()).unwrap()
            };

            let tx_desc = NonEmptyString::from_str(&format!(
                "{} ({}/{})",
                new_inst.description.as_str(),
                i,
                new_inst.installments_count
            ))
            .unwrap();

            let new_tx = crate::models::transaction::NewTransaction {
                category_id: new_inst.category_id,
                source: crate::models::types::TransactionSource::CreditCard {
                    credit_card_id: new_inst.credit_card_id,
                },
                transaction_type: crate::models::types::TransactionType::Expense, // Cartão sempre gera Expense
                amount: tx_amount,
                date: tx_date,
                description: tx_desc,
                installment_id: Some(installment.id),
                installment_number: Some(i),
                tags: vec![],
            };

            // Reutilizamos a lógica da transação, passando nossa conexão &mut *tx
            crate::models::transaction::Transaction::insert(&mut *tx, new_tx).await?;
        }

        tx.commit().await?;
        Ok(installment)
    }

    pub async fn find_all(
        pool: &sqlx::PgPool,
        limit: Option<usize>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let limit = limit.unwrap_or(100) as i64;
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM installments
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: i32) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM installments
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &sqlx::PgPool, id: i32) -> Result<bool, sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Verificar se já existe transação atrelada a uma fatura PAGA
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
            return Err(sqlx::Error::Protocol(
                "Não é possível deletar um parcelamento onde alguma fatura atrelada já foi fechada ou paga.".into()
            ));
        }

        // As faturas agora são calculadas dinamicamente com base nas transações.
        // Apenas deletamos as transações da fatura.

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
        number: i16,
        new_amount: PositiveAmount,
    ) -> Result<crate::models::transaction::Transaction, sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Achar a transação
        let old_tx = sqlx::query_as::<_, crate::models::transaction::Transaction>(
            r#"SELECT * FROM transactions WHERE installment_id = $1 AND installment_number = $2"#,
        )
        .bind(installment_id)
        .bind(number)
        .fetch_optional(&mut *tx)
        .await?;

        let old_tx = match old_tx {
            Some(t) => t,
            None => return Err(sqlx::Error::RowNotFound),
        };

        if let Some(inv_id) = old_tx.invoice_id {
            // Checar se a fatura não está fechada/paga
            let status = sqlx::query_scalar::<_, crate::models::types::InvoiceStatus>(
                "SELECT status FROM invoices WHERE id = $1",
            )
            .bind(inv_id)
            .fetch_one(&mut *tx)
            .await?;

            if status != crate::models::types::InvoiceStatus::Open {
                return Err(sqlx::Error::Protocol(
                    "Fatura já fechada ou paga. Ajuste bloqueado.".into(),
                ));
            }
            // Não precisamos mais atualizar a fatura manualmente, o valor total é a soma on-the-fly.
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
}
