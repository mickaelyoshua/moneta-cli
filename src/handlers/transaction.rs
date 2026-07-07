use crate::commands::transaction::{AddTransactionArgs, TransactionError};
use crate::context::AppContext;
use crate::models::types::TransactionType;

pub async fn add(ctx: &AppContext, args: AddTransactionArgs) -> Result<(), TransactionError> {
    let new_tx: crate::models::transaction::NewTransaction = args.try_into()?;
    let tx = crate::models::transaction::Transaction::insert(&ctx.db.pool, new_tx).await?;

    if ctx.json_output {
        println!("{}", serde_json::to_string(&tx)?);
    } else {
        println!(
            "Transaction #{} of {} added on {}.",
            tx.id,
            tx.amount.as_decimal(),
            tx.date
        );
    }
    Ok(())
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<(), TransactionError> {
    let txs = crate::models::transaction::Transaction::find_all(&ctx.db.pool, limit).await?;

    if ctx.json_output {
        println!("{}", serde_json::to_string(&txs)?);
    } else {
        for tx in txs {
            let source_id = match tx.source {
                crate::models::types::TransactionSource::Account { account_id } => {
                    format!("Acc #{}", account_id)
                }
                crate::models::types::TransactionSource::CreditCard { credit_card_id } => {
                    format!("Card #{}", credit_card_id)
                }
            };
            println!(
                "[{}] #{} - {} {} ({}) -> {}",
                tx.date,
                tx.id,
                if tx.transaction_type == TransactionType::Income {
                    "+"
                } else {
                    "-"
                },
                tx.amount.as_decimal(),
                source_id,
                tx.description.as_str()
            );
        }
    }
    Ok(())
}
