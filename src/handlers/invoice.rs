use crate::{
    commands::invoice::InvoiceError,
    context::AppContext,
    models::invoice::Invoice,
    models::types::{Month, Year},
};

pub async fn view(
    ctx: &AppContext,
    credit_card_id: i32,
    limit: Option<usize>,
) -> Result<Vec<Invoice>, InvoiceError> {
    let invoices = Invoice::find_all_by_card(&ctx.db.pool, credit_card_id, limit).await?;
    Ok(invoices)
}

pub async fn close(
    ctx: &AppContext,
    credit_card_id: i32,
    month: Month,
    year: Year,
) -> Result<Invoice, InvoiceError> {
    let invoice = Invoice::close(&ctx.db.pool, credit_card_id, month, year).await?;
    Ok(invoice)
}

pub async fn reopen(
    ctx: &AppContext,
    credit_card_id: i32,
    month: Month,
    year: Year,
) -> Result<Invoice, InvoiceError> {
    let invoice = Invoice::reopen(&ctx.db.pool, credit_card_id, month, year).await?;
    Ok(invoice)
}

pub async fn pay(
    ctx: &AppContext,
    credit_card_id: i32,
    month: Month,
    year: Year,
    account_id: i32,
) -> Result<Invoice, InvoiceError> {
    let invoice = Invoice::pay(&ctx.db.pool, credit_card_id, month, year, account_id).await?;
    Ok(invoice)
}
