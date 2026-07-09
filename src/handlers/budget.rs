use crate::context::AppContext;
use crate::models::types::{BudgetPeriod, PositiveAmount};
use chrono::Local;

#[derive(thiserror::Error, Debug)]
pub enum BudgetError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

pub async fn add(
    ctx: &AppContext,
    category_id: Option<i32>,
    tag_id: Option<i32>,
    limit: PositiveAmount,
    period: BudgetPeriod,
) -> Result<(), BudgetError> {
    let budget =
        crate::models::budget::Budget::insert(&ctx.db.pool, category_id, tag_id, limit, period)
            .await?;
    crate::handlers::render_success(ctx, &budget);
    Ok(())
}

pub async fn list(ctx: &AppContext, date: Option<chrono::NaiveDate>) -> Result<(), BudgetError> {
    let ref_date = date.unwrap_or_else(|| Local::now().naive_local().date());
    let budgets = crate::models::budget::Budget::list_with_spend(&ctx.db.pool, ref_date).await?;
    crate::handlers::render_success(ctx, &budgets);
    Ok(())
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<(), BudgetError> {
    crate::models::budget::Budget::delete(&ctx.db.pool, id).await?;
    crate::handlers::render_success(ctx, &serde_json::json!({ "deleted": id }));
    Ok(())
}
