use super::types::NonEmptyString;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: NonEmptyString,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TransactionTag {
    pub transaction_id: i32,
    pub tag_id: i32,
}

impl Tag {
    pub async fn resolve_names(
        conn: &mut sqlx::PgConnection,
        names: &[NonEmptyString],
    ) -> Result<Vec<i32>, sqlx::Error> {
        if names.is_empty() {
            return Ok(vec![]);
        }

        let mut ids = Vec::new();
        for name in names {
            let record = sqlx::query!(
                "INSERT INTO tags (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
                name.as_str()
            )
            .fetch_one(&mut *conn)
            .await?;
            ids.push(record.id);
        }

        Ok(ids)
    }
}
