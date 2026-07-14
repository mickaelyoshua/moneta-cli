use crate::{
    commands::installment::{AddInstallmentArgs, AdjustInstallmentArgs, InstallmentError},
    context::AppContext,
    models::installment::{Installment, InstallmentDetails, NewInstallment},
};
use std::str::FromStr;

pub async fn add(
    ctx: &AppContext,
    args: AddInstallmentArgs,
) -> Result<Installment, InstallmentError> {
    let new_inst: NewInstallment = args.try_into()?;
    let inst = Installment::insert(&ctx.db.pool, new_inst).await?;
    Ok(inst)
}

pub async fn list(
    ctx: &AppContext,
    limit: Option<usize>,
) -> Result<Vec<Installment>, InstallmentError> {
    let installments = Installment::find_all(&ctx.db.pool, limit).await?;
    Ok(installments)
}

pub async fn show(ctx: &AppContext, id: i32) -> Result<InstallmentDetails, InstallmentError> {
    let inst = Installment::find_by_id(&ctx.db.pool, id).await?;
    let txs = sqlx::query_as::<_, crate::models::transaction::Transaction>(
        "SELECT * FROM transactions WHERE installment_id = $1 ORDER BY installment_number",
    )
    .bind(id)
    .fetch_all(&ctx.db.pool)
    .await?;

    Ok(InstallmentDetails {
        installment: inst,
        transactions: txs,
    })
}

pub async fn adjust(
    ctx: &AppContext,
    args: AdjustInstallmentArgs,
) -> Result<crate::models::transaction::Transaction, InstallmentError> {
    let new_amount = crate::models::types::PositiveAmount::from_str(&args.amount)
        .map_err(|e| InstallmentError::Parse(e.to_string()))?;

    let tx = Installment::adjust(&ctx.db.pool, args.id, args.number, new_amount).await?;
    Ok(tx)
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<serde_json::Value, InstallmentError> {
    Installment::delete(&ctx.db.pool, id).await?;
    Ok(serde_json::json!({ "deleted": true, "id": id }))
}
