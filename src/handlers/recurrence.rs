use crate::context::AppContext;
use crate::models::recurrence::{NewRecurrence, Recurrence, UpdateRecurrencePayload};
use chrono::Local;

#[derive(thiserror::Error, Debug)]
pub enum RecurrenceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Model error: {0}")]
    Model(#[from] crate::models::ModelError),
}

pub async fn add(ctx: &AppContext, new_rec: NewRecurrence) -> Result<Recurrence, RecurrenceError> {
    let rec = Recurrence::insert(&ctx.db.pool, new_rec).await?;
    Ok(rec)
}

pub async fn list(ctx: &AppContext) -> Result<Vec<Recurrence>, RecurrenceError> {
    let recs = Recurrence::find_all(&ctx.db.pool).await?;
    Ok(recs)
}

pub async fn update(
    ctx: &AppContext,
    id: i32,
    payload: UpdateRecurrencePayload,
) -> Result<Recurrence, RecurrenceError> {
    let rec = Recurrence::update(&ctx.db.pool, id, payload).await?;
    Ok(rec)
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<serde_json::Value, RecurrenceError> {
    Recurrence::delete(&ctx.db.pool, id).await?;
    Ok(serde_json::json!({ "deleted": id }))
}

pub async fn sync(
    ctx: &AppContext,
    date: Option<chrono::NaiveDate>,
) -> Result<serde_json::Value, RecurrenceError> {
    let ref_date = date.unwrap_or_else(|| Local::now().naive_local().date());
    let inserted = Recurrence::sync_all(&ctx.db.pool, ref_date).await?;
    Ok(serde_json::json!({ "inserted": inserted }))
}
