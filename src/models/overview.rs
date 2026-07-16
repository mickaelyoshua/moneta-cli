use super::budget::BudgetWithSpend;
use super::types::{InvoiceStatus, Month, Year};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AccountOverview {
    pub id: i32,
    pub name: String,
    pub balance: Decimal,
}

#[derive(Debug, Serialize)]
pub struct InvoiceOverview {
    pub month: Month,
    pub year: Year,
    pub status: InvoiceStatus,
    pub amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct CreditCardOverview {
    pub id: i32,
    pub name: String,
    pub limit: Decimal,
    pub used: Decimal,
    pub available: Decimal,
    pub open_invoices: Vec<InvoiceOverview>,
}

#[derive(Debug, Serialize)]
pub struct OverviewResponse {
    pub reference_date: NaiveDate,
    pub accounts: Vec<AccountOverview>,
    pub credit_cards: Vec<CreditCardOverview>,
    pub budgets: Vec<BudgetWithSpend>,
}

impl OverviewResponse {
    pub async fn generate(
        pool: &sqlx::PgPool,
        date: Option<NaiveDate>,
    ) -> Result<Self, crate::models::ModelError> {
        let reference_date = date.unwrap_or_else(|| chrono::Utc::now().naive_local().date());

        let accounts = super::account::Account::find_all(pool, None).await?;
        let mut acc_res = Vec::new();
        for acc in accounts.iter().filter(|acc| acc.active) {
            let bal = super::account::Account::balance(pool, acc.id).await?;
            acc_res.push(AccountOverview {
                id: acc.id,
                name: acc.name.as_str().to_string(),
                balance: bal.balance,
            });
        }

        let cards = super::credit_card::CreditCard::find_all(pool, None).await?;
        let mut card_res = Vec::new();
        for card in cards.iter().filter(|card| card.active) {
            let used = super::credit_card::CreditCard::used_limit(pool, card.id).await?;
            let limit = card.credit_limit.as_decimal();
            let available = if limit > used {
                limit - used
            } else {
                Decimal::ZERO
            };

            let open_invoices =
                super::invoice::Invoice::get_open_overviews_by_card(pool, card.id).await?;

            card_res.push(CreditCardOverview {
                id: card.id,
                name: card.name.as_str().to_string(),
                limit,
                used,
                available,
                open_invoices,
            });
        }

        let budgets = super::budget::Budget::list_with_spend(pool, reference_date).await?;

        Ok(OverviewResponse {
            reference_date,
            accounts: acc_res,
            credit_cards: card_res,
            budgets,
        })
    }
}
