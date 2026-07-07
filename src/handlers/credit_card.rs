use crate::{
    commands::credit_card::{AddCreditCardArgs, CreditCardError},
    context::AppContext,
    models::credit_card::{CreditCard, NewCreditCard},
};

pub async fn add(ctx: &AppContext, args: AddCreditCardArgs) -> Result<(), CreditCardError> {
    let new_card: NewCreditCard = args.try_into()?;

    let card = sqlx::query_as::<_, CreditCard>(
        r#"
        INSERT INTO credit_cards (account_id, name, credit_limit, billing_day, due_day, active)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(new_card.account_id)
    .bind(new_card.name.as_str())
    .bind(new_card.credit_limit)
    .bind(new_card.billing_day.as_i16())
    .bind(new_card.due_day.as_i16())
    .bind(new_card.active)
    .fetch_one(&ctx.db.pool)
    .await?;

    crate::handlers::render_success(ctx, &card);

    Ok(())
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<(), CreditCardError> {
    let limit = limit.unwrap_or(100) as i64;
    let cards = sqlx::query_as::<_, CreditCard>(
        r#"
        SELECT * FROM credit_cards
        ORDER BY created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(&ctx.db.pool)
    .await?;

    crate::handlers::render_success(ctx, &cards);

    Ok(())
}
