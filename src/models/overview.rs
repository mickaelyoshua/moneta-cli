use super::budget::BudgetWithSpend;
use super::types::InvoiceStatus;
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
    pub month: i16,
    pub year: i16,
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
