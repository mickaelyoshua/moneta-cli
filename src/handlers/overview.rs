use crate::{context::AppContext, error::AppError, models::overview::OverviewResponse};
use chrono::NaiveDate;

pub async fn handle_overview(
    ctx: &AppContext,
    date: Option<NaiveDate>,
) -> Result<OverviewResponse, AppError> {
    let res = OverviewResponse::generate(&ctx.db.pool, date).await?;
    Ok(res)
}
