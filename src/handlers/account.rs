use crate::{
    commands::account::{AccountError, AddAccountArgs},
    context::AppContext,
    models::account::{Account, NewAccount},
};

pub async fn add(ctx: &AppContext, args: AddAccountArgs) -> Result<(), AccountError> {
    let new_acc: NewAccount = args.try_into()?;

    let acc = sqlx::query_as::<_, Account>(
        r#"
        INSERT INTO accounts (name, account_type, has_debit_card, active)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(new_acc.name.as_str())
    .bind(new_acc.account_type)
    .bind(new_acc.has_debit_card)
    .bind(new_acc.active)
    .fetch_one(&ctx.db.pool)
    .await?;

    crate::handlers::render_success(ctx, &acc);

    Ok(())
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<(), AccountError> {
    let limit = limit.unwrap_or(100) as i64;
    let accounts = sqlx::query_as::<_, Account>(
        r#"
        SELECT * FROM accounts
        ORDER BY created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(&ctx.db.pool)
    .await?;

    crate::handlers::render_success(ctx, &accounts);

    Ok(())
}
