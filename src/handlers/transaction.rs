use crate::commands::transaction::{AddTransactionArgs, TransactionError};
use crate::context::AppContext;

pub async fn add(ctx: &AppContext, args: AddTransactionArgs) -> Result<(), TransactionError> {
    let new_tx: crate::models::transaction::NewTransaction = args.try_into()?;
    let tx = crate::models::transaction::Transaction::insert(&ctx.db.pool, new_tx).await?;

    crate::handlers::render_success(ctx, &tx);
    Ok(())
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<(), TransactionError> {
    let txs = crate::models::transaction::Transaction::find_all(&ctx.db.pool, limit).await?;

    crate::handlers::render_success(ctx, &txs);
    Ok(())
}
