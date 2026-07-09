use crate::context::AppContext;
use chrono::Local;

#[derive(thiserror::Error, Debug)]
pub enum RecurrenceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

pub async fn add(
    ctx: &AppContext,
    new_rec: crate::models::recurrence::NewRecurrence,
) -> Result<(), RecurrenceError> {
    let rec = crate::models::recurrence::Recurrence::insert(&ctx.db.pool, new_rec).await?;
    crate::handlers::render_success(ctx, &rec);
    Ok(())
}

pub async fn list(ctx: &AppContext) -> Result<(), RecurrenceError> {
    let recs = crate::models::recurrence::Recurrence::find_all(&ctx.db.pool).await?;
    crate::handlers::render_success(ctx, &recs);
    Ok(())
}

pub async fn update(
    ctx: &AppContext,
    id: i32,
    payload: crate::models::recurrence::UpdateRecurrencePayload,
) -> Result<(), RecurrenceError> {
    let rec = crate::models::recurrence::Recurrence::update(&ctx.db.pool, id, payload).await?;
    crate::handlers::render_success(ctx, &rec);
    Ok(())
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<(), RecurrenceError> {
    crate::models::recurrence::Recurrence::delete(&ctx.db.pool, id).await?;
    crate::handlers::render_success(ctx, &serde_json::json!({ "deleted": id }));
    Ok(())
}

pub async fn sync(
    ctx: &AppContext,
    date: Option<chrono::NaiveDate>,
) -> Result<(), RecurrenceError> {
    let ref_date = date.unwrap_or_else(|| Local::now().naive_local().date());
    let inserted = crate::models::recurrence::Recurrence::sync_all(&ctx.db.pool, ref_date).await?;
    crate::handlers::render_success(ctx, &serde_json::json!({ "inserted": inserted }));
    Ok(())
}
