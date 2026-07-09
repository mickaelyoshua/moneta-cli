use super::types::{
    NonEmptyString, PositiveAmount, TransactionSource, TransactionStatus, TransactionType,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::Connection;

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub id: i32,
    pub category_id: Option<i32>,

    #[serde(flatten)]
    pub source: TransactionSource,

    pub invoice_id: Option<i32>,
    pub installment_id: Option<i32>,
    pub recurrence_id: Option<i32>,

    pub transaction_type: TransactionType,
    pub amount: PositiveAmount,
    pub date: NaiveDate,
    pub description: NonEmptyString,
    pub installment_number: Option<i16>,
    pub status: TransactionStatus,

    pub tags: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewTransaction {
    pub category_id: Option<i32>,
    pub source: TransactionSource,
    pub transaction_type: TransactionType,
    pub amount: PositiveAmount,
    pub date: NaiveDate,
    pub description: NonEmptyString,
    pub installment_id: Option<i32>,
    pub installment_number: Option<i16>,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct UpdateTransactionPayload {
    pub account_id: Option<i32>,
    pub credit_card_id: Option<i32>,
    pub category_id: Option<i32>,
    pub transaction_type: Option<TransactionType>,
    pub amount: Option<PositiveAmount>,
    pub date: Option<chrono::NaiveDate>,
    pub description: Option<NonEmptyString>,
}

impl Transaction {
    pub async fn insert(conn: &mut sqlx::PgConnection, new_tx: NewTransaction) -> Result<Self, sqlx::Error> {
        let mut account_id = None;
        let mut credit_card_id = None;
        let mut invoice_id = None;

        match new_tx.source {
            TransactionSource::Account { account_id: id } => {
                account_id = Some(id);
            }
            TransactionSource::CreditCard { credit_card_id: id } => {
                credit_card_id = Some(id);
                // Registrar: Find or create invoice for this transaction
                let invoice =
                    crate::models::invoice::Invoice::find_or_create_for_date(&mut *conn, id, new_tx.date)
                        .await?;

                if invoice.status != crate::models::types::InvoiceStatus::Open {
                    return Err(sqlx::Error::Protocol(
                        "Não é possível adicionar transação a uma fatura fechada ou paga."
                            .to_string(),
                    ));
                }

                invoice_id = Some(invoice.id);
            }
        };

        let mut row = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions (
                category_id, account_id, credit_card_id, invoice_id,
                transaction_type, amount, date, description, installment_id, installment_number
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(new_tx.category_id)
        .bind(account_id)
        .bind(credit_card_id)
        .bind(invoice_id)
        .bind(new_tx.transaction_type)
        .bind(new_tx.amount.as_decimal())
        .bind(new_tx.date)
        .bind(new_tx.description.as_str())
        .bind(new_tx.installment_id)
        .bind(new_tx.installment_number)
        .fetch_one(&mut *conn)
        .await?;

        if !new_tx.tags.is_empty() {
            let tag_ids = crate::models::tag::Tag::resolve_names(&mut *conn, &new_tx.tags).await?;
            for tag_id in tag_ids {
                sqlx::query!(
                    "INSERT INTO transaction_tags (transaction_id, tag_id) VALUES ($1, $2)",
                    row.id,
                    tag_id
                )
                .execute(&mut *conn)
                .await?;
            }
            row.tags = new_tx.tags;
        }

        Ok(row)
    }

    pub async fn find_all(
        pool: &sqlx::PgPool,
        limit: Option<usize>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let limit = limit.unwrap_or(50) as i64;
        let rows = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT t.*, COALESCE(array_agg(tg.name) FILTER (WHERE tg.name IS NOT NULL), '{}') AS tags
            FROM transactions t
            LEFT JOIN transaction_tags tt ON tt.transaction_id = t.id
            LEFT JOIN tags tg ON tg.id = tt.tag_id
            GROUP BY t.id
            ORDER BY t.date DESC, t.created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn find_by_id(
        pool: &sqlx::PgPool,
        id: i32,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT t.*, COALESCE(array_agg(tg.name) FILTER (WHERE tg.name IS NOT NULL), '{}') AS tags
            FROM transactions t
            LEFT JOIN transaction_tags tt ON tt.transaction_id = t.id
            LEFT JOIN tags tg ON tg.id = tt.tag_id
            WHERE t.id = $1
            GROUP BY t.id
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(
        pool: &sqlx::PgPool,
        id: i32,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        let old_tx = sqlx::query_as::<_, Self>(
            "SELECT * FROM transactions WHERE id = $1 FOR UPDATE"
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        let old_tx = match old_tx {
            Some(t) => t,
            None => return Err(sqlx::Error::RowNotFound),
        };

        if old_tx.installment_id.is_some() {
            return Err(sqlx::Error::Protocol(
                "Esta transação pertence a um parcelamento. Utilize 'moneta installment delete' para apagá-la.".into()
            ));
        }

        if let Some(inv_id) = old_tx.invoice_id {
            let status = sqlx::query_scalar::<_, crate::models::types::InvoiceStatus>(
                "SELECT status FROM invoices WHERE id = $1"
            )
            .bind(inv_id)
            .fetch_one(&mut *tx)
            .await?;

            if status != crate::models::types::InvoiceStatus::Open {
                return Err(sqlx::Error::Protocol(
                    "Não é possível deletar transação de uma fatura fechada ou paga.".into()
                ));
            }
        }

        sqlx::query!("DELETE FROM transactions WHERE id = $1", id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update(
        conn: &mut sqlx::PgConnection,
        id: i32,
        payload: UpdateTransactionPayload,
    ) -> Result<Self, sqlx::Error> {
        let mut db_tx = conn.begin().await?;

        let old_tx = sqlx::query_as::<_, Self>(
            "SELECT * FROM transactions WHERE id = $1 FOR UPDATE"
        )
        .bind(id)
        .fetch_optional(&mut *db_tx)
        .await?;

        let old_tx = match old_tx {
            Some(t) => t,
            None => return Err(sqlx::Error::RowNotFound),
        };

        if old_tx.installment_id.is_some() {
            return Err(sqlx::Error::Protocol(
                "Esta transação pertence a um parcelamento. Utilize 'moneta installment adjust' para modificar o valor.".into()
            ));
        }

        // Validação da Invoice Original
        if let Some(inv_id) = old_tx.invoice_id {
            let status = sqlx::query_scalar::<_, crate::models::types::InvoiceStatus>(
                "SELECT status FROM invoices WHERE id = $1"
            )
            .bind(inv_id)
            .fetch_one(&mut *db_tx)
            .await?;

            if status != crate::models::types::InvoiceStatus::Open {
                return Err(sqlx::Error::Protocol(
                    "Não é possível alterar transação de uma fatura fechada ou paga.".into()
                ));
            }
        }

        let mut next_account_id = match old_tx.source {
            TransactionSource::Account { account_id } => Some(account_id),
            _ => None,
        };
        let mut next_credit_card_id = match old_tx.source {
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

        let next_date = payload.date.unwrap_or(old_tx.date);
        let mut next_invoice_id = None;

        // Se o destino for Cartão, temos que achar a Invoice alvo e verificar se está aberta
        if let Some(cc_id) = next_credit_card_id {
            let invoice = crate::models::invoice::Invoice::find_or_create_for_date(&mut *db_tx, cc_id, next_date).await?;
            if invoice.status != crate::models::types::InvoiceStatus::Open {
                return Err(sqlx::Error::Protocol(
                    "A fatura de destino já está fechada ou paga.".into()
                ));
            }
            next_invoice_id = Some(invoice.id);
        }

        let category_id = payload.category_id.or(old_tx.category_id);
        let transaction_type = payload.transaction_type.unwrap_or(old_tx.transaction_type);
        let amount = payload.amount.unwrap_or(old_tx.amount);
        let description = payload.description.unwrap_or(old_tx.description);

        let row = sqlx::query_as::<_, Self>(
            r#"
            UPDATE transactions
            SET 
                account_id = $1, credit_card_id = $2, invoice_id = $3, category_id = $4,
                transaction_type = $5, amount = $6, date = $7, description = $8,
                updated_at = NOW()
            WHERE id = $9
            RETURNING *
            "#
        )
        .bind(next_account_id)
        .bind(next_credit_card_id)
        .bind(next_invoice_id)
        .bind(category_id)
        .bind(transaction_type)
        .bind(amount.as_decimal())
        .bind(next_date)
        .bind(description.as_str())
        .bind(id)
        .fetch_one(&mut *db_tx)
        .await?;

        db_tx.commit().await?;

        Ok(row)
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
            invoice_id: row.try_get("invoice_id")?,
            installment_id: row.try_get("installment_id")?,
            recurrence_id: row.try_get("recurrence_id")?,
            transaction_type: row.try_get("transaction_type")?,
            amount: row.try_get("amount")?,
            date: row.try_get("date")?,
            description: row.try_get("description")?,
            installment_number: row.try_get("installment_number")?,
            status: row.try_get("status")?,
            tags: row.try_get("tags").unwrap_or_default(),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
