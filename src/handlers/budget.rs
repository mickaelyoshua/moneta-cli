use crate::context::AppContext;
use crate::models::types::{BudgetPeriod, PositiveAmount};
use chrono::Local;

#[derive(thiserror::Error, Debug)]
pub enum BudgetError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Model error: {0}")]
    Model(#[from] crate::models::ModelError),
}

pub async fn add(
    ctx: &AppContext,
    category_id: Option<i32>,
    tag_id: Option<i32>,
    limit: PositiveAmount,
    period: BudgetPeriod,
) -> Result<crate::models::budget::Budget, BudgetError> {
    let budget =
        crate::models::budget::Budget::insert(&ctx.db.pool, category_id, tag_id, limit, period)
            .await?;
    Ok(budget)
}

pub async fn list(
    ctx: &AppContext,
    date: Option<chrono::NaiveDate>,
) -> Result<Vec<crate::models::budget::BudgetWithSpend>, BudgetError> {
    let ref_date = date.unwrap_or_else(|| Local::now().naive_local().date());
    let budgets = crate::models::budget::Budget::list_with_spend(&ctx.db.pool, ref_date).await?;
    Ok(budgets)
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<serde_json::Value, BudgetError> {
    crate::models::budget::Budget::delete(&ctx.db.pool, id).await?;
    Ok(serde_json::json!({ "deleted": id }))
}
