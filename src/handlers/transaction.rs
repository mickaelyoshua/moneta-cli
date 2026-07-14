use crate::commands::transaction::{AddTransactionArgs, TransactionError};
use crate::context::AppContext;
use crate::models::transaction::{NewTransaction, Transaction, UpdateTransactionPayload};

pub async fn add(
    ctx: &AppContext,
    args: AddTransactionArgs,
) -> Result<Transaction, TransactionError> {
    let new_tx: NewTransaction = args.try_into()?;
    let mut conn = ctx.db.pool.acquire().await?;
    let tx = Transaction::insert(&mut conn, new_tx).await?;
    Ok(tx)
}

pub async fn list(
    ctx: &AppContext,
    limit: Option<usize>,
) -> Result<Vec<Transaction>, TransactionError> {
    let txs = Transaction::find_all(&ctx.db.pool, limit).await?;
    Ok(txs)
}

pub async fn show(ctx: &AppContext, id: i32) -> Result<Transaction, TransactionError> {
    let tx = Transaction::find_by_id(&ctx.db.pool, id).await?;
    Ok(tx)
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<serde_json::Value, TransactionError> {
    Transaction::delete(&ctx.db.pool, id).await?;
    Ok(serde_json::json!({ "deleted": id }))
}

pub async fn update(
    ctx: &AppContext,
    args: crate::commands::transaction::UpdateTransactionArgs,
) -> Result<Transaction, TransactionError> {
    let payload = UpdateTransactionPayload {
        account_id: args.account_id,
        credit_card_id: args.credit_card_id,
        category_id: args.category_id,
        transaction_type: args.transaction_type,
        amount: args.amount,
        date: args.date,
        description: args.description,
    };

    let mut conn = ctx.db.pool.acquire().await?;
    let tx = Transaction::update(&mut conn, args.id, payload).await?;
    Ok(tx)
}
