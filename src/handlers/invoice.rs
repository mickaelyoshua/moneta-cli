use crate::{commands::invoice::InvoiceError, context::AppContext, models::invoice::Invoice};

pub async fn view(
    ctx: &AppContext,
    credit_card_id: i32,
    limit: Option<usize>,
) -> Result<(), InvoiceError> {
    let invoices = Invoice::find_all_by_card(&ctx.db.pool, credit_card_id, limit).await?;

    crate::handlers::render_success(ctx, &invoices);

    Ok(())
}

pub async fn close(
    ctx: &AppContext,
    credit_card_id: i32,
    month: i16,
    year: i16,
) -> Result<(), InvoiceError> {
    let invoice = Invoice::close(&ctx.db.pool, credit_card_id, month, year).await?;

    crate::handlers::render_success(ctx, &invoice);

    Ok(())
}

pub async fn reopen(
    ctx: &AppContext,
    credit_card_id: i32,
    month: i16,
    year: i16,
) -> Result<(), InvoiceError> {
    let invoice = Invoice::reopen(&ctx.db.pool, credit_card_id, month, year).await?;

    crate::handlers::render_success(ctx, &invoice);

    Ok(())
}

pub async fn pay(
    ctx: &AppContext,
    credit_card_id: i32,
    month: i16,
    year: i16,
    account_id: i32,
) -> Result<(), InvoiceError> {
    let invoice = Invoice::pay(&ctx.db.pool, credit_card_id, month, year, account_id).await?;

    crate::handlers::render_success(ctx, &invoice);

    Ok(())
}
