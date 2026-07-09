use crate::{
    context::AppContext,
    error::AppError,
    models::{
        account::Account,
        budget::Budget,
        credit_card::CreditCard,
        overview::{AccountOverview, CreditCardOverview, InvoiceOverview, OverviewResponse},
        types::InvoiceStatus,
    },
};
use chrono::NaiveDate;
use rust_decimal::Decimal;

pub async fn handle_overview(
    ctx: &AppContext,
    date: Option<NaiveDate>,
) -> Result<OverviewResponse, AppError> {
    let pool = &ctx.db.pool;
    let reference_date = date.unwrap_or_else(|| chrono::Utc::now().naive_local().date());

    let accounts = Account::find_all(pool, None).await?;
    let mut acc_res = Vec::new();
    for acc in accounts {
        if !acc.active {
            continue;
        }
        let bal = Account::balance(pool, acc.id).await?;
        acc_res.push(AccountOverview {
            id: acc.id,
            name: acc.name.as_str().to_string(),
            balance: bal.balance,
        });
    }

    let cards = CreditCard::find_all(pool, None).await?;
    let mut card_res = Vec::new();
    for card in cards {
        if !card.active {
            continue;
        }

        let used = CreditCard::used_limit(pool, card.id).await?;
        let limit = card.credit_limit.as_decimal();
        let available = if limit > used {
            limit - used
        } else {
            Decimal::ZERO
        };

        let open_invoices_rows = sqlx::query!(
            r#"
            SELECT 
                i.month, 
                i.year, 
                i.status as "status: InvoiceStatus", 
                COALESCE(SUM(
                    CASE 
                        WHEN t.transaction_type = 'income' THEN -t.amount 
                        ELSE t.amount 
                    END
                ), 0) as total
            FROM invoices i
            LEFT JOIN transactions t ON t.invoice_id = i.id
            WHERE i.credit_card_id = $1
            GROUP BY i.id
            HAVING COALESCE(SUM(
                CASE 
                    WHEN t.transaction_type = 'income' THEN -t.amount 
                    ELSE t.amount 
                END
            ), 0) != 0
            ORDER BY i.year ASC, i.month ASC
            "#,
            card.id
        )
        .fetch_all(pool)
        .await?;

        let mut open_invoices = Vec::new();
        for row in open_invoices_rows {
            open_invoices.push(InvoiceOverview {
                month: row.month,
                year: row.year,
                status: row.status,
                amount: row.total.unwrap_or(Decimal::ZERO),
            });
        }

        card_res.push(CreditCardOverview {
            id: card.id,
            name: card.name.as_str().to_string(),
            limit,
            used,
            available,
            open_invoices,
        });
    }

    let budgets = Budget::list_with_spend(pool, reference_date).await?;

    Ok(OverviewResponse {
        reference_date,
        accounts: acc_res,
        credit_cards: card_res,
        budgets,
    })
}
