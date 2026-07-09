use crate::commands::transaction::{AddTransactionArgs, TransactionError};
use crate::context::AppContext;

pub async fn add(ctx: &AppContext, args: AddTransactionArgs) -> Result<(), TransactionError> {
    let new_tx: crate::models::transaction::NewTransaction = args.try_into()?;
    let mut conn = ctx.db.pool.acquire().await?;
    let tx = crate::models::transaction::Transaction::insert(&mut *conn, new_tx).await?;

    crate::handlers::render_success(ctx, &tx);
    Ok(())
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<(), TransactionError> {
    let txs = crate::models::transaction::Transaction::find_all(&ctx.db.pool, limit).await?;

    crate::handlers::render_success(ctx, &txs);
    Ok(())
}

pub async fn show(ctx: &AppContext, id: i32) -> Result<(), TransactionError> {
    let tx = crate::models::transaction::Transaction::find_by_id(&ctx.db.pool, id).await?;
    crate::handlers::render_success(ctx, &tx);
    Ok(())
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<(), TransactionError> {
    crate::models::transaction::Transaction::delete(&ctx.db.pool, id).await?;
    crate::handlers::render_success(ctx, &serde_json::json!({ "deleted": id }));
    Ok(())
}

pub async fn update(
    ctx: &AppContext,
    args: crate::commands::transaction::UpdateTransactionArgs,
) -> Result<(), TransactionError> {
    // We construct a struct to pass only the optional fields
    let payload = crate::models::transaction::UpdateTransactionPayload {
        account_id: args.account_id,
        credit_card_id: args.credit_card_id,
        category_id: args.category_id,
        transaction_type: args.transaction_type,
        amount: args.amount,
        date: args.date,
        description: args.description,
    };

    let mut conn = ctx.db.pool.acquire().await?;
    let tx = crate::models::transaction::Transaction::update(&mut *conn, args.id, payload).await?;
    crate::handlers::render_success(ctx, &tx);
    Ok(())
}
