use crate::{
    commands::installment::{AddInstallmentArgs, AdjustInstallmentArgs, InstallmentError},
    context::AppContext,
    models::installment::{Installment, NewInstallment},
};
use std::str::FromStr;

pub async fn add(ctx: &AppContext, args: AddInstallmentArgs) -> Result<(), InstallmentError> {
    let new_inst: NewInstallment = args.try_into()?;
    let inst = Installment::insert(&ctx.db.pool, new_inst).await?;
    crate::handlers::render_success(ctx, &inst);
    Ok(())
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<(), InstallmentError> {
    let installments = Installment::find_all(&ctx.db.pool, limit).await?;
    crate::handlers::render_success(ctx, &installments);
    Ok(())
}

pub async fn show(ctx: &AppContext, id: i32) -> Result<(), InstallmentError> {
    let inst = Installment::find_by_id(&ctx.db.pool, id).await?;
    // We can also fetch the transactions associated
    let txs = sqlx::query_as::<_, crate::models::transaction::Transaction>(
        "SELECT * FROM transactions WHERE installment_id = $1 ORDER BY installment_number"
    )
    .bind(id)
    .fetch_all(&ctx.db.pool)
    .await?;

    if ctx.json_output {
        crate::handlers::render_success(ctx, &serde_json::json!({
            "installment": inst,
            "transactions": txs
        }));
    } else {
        println!("{:#?}", inst);
        println!("Transações:");
        for tx in txs {
            println!("- Parcela {:?}: {} ({})", tx.installment_number, tx.amount.as_decimal(), tx.date);
        }
    }
    Ok(())
}

pub async fn adjust(ctx: &AppContext, args: AdjustInstallmentArgs) -> Result<(), InstallmentError> {
    let new_amount = crate::models::types::PositiveAmount::from_str(&args.amount)
        .map_err(|e| InstallmentError::Parse(e.to_string()))?;
    
    let tx = Installment::adjust(&ctx.db.pool, args.id, args.number, new_amount).await?;
    crate::handlers::render_success(ctx, &tx);
    Ok(())
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<(), InstallmentError> {
    Installment::delete(&ctx.db.pool, id).await?;
    crate::handlers::render_success(ctx, &serde_json::json!({ "deleted": true, "id": id }));
    Ok(())
}
