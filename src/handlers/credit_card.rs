use crate::{
    commands::credit_card::{AddCreditCardArgs, CreditCardError},
    context::AppContext,
    models::credit_card::{CreditCard, NewCreditCard},
};

pub async fn add(ctx: &AppContext, args: AddCreditCardArgs) -> Result<(), CreditCardError> {
    let new_card: NewCreditCard = args.try_into()?;
    let card = CreditCard::insert(&ctx.db.pool, new_card).await?;

    crate::handlers::render_success(ctx, &card);

    Ok(())
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<(), CreditCardError> {
    let cards = CreditCard::find_all(&ctx.db.pool, limit).await?;

    crate::handlers::render_success(ctx, &cards);

    Ok(())
}

pub async fn update(
    ctx: &AppContext,
    args: crate::commands::credit_card::UpdateCreditCardArgs,
) -> Result<(), CreditCardError> {
    let mut card = CreditCard::find_by_id(&ctx.db.pool, args.id).await?;

    if let Some(name) = args.name {
        card.name = name;
    }
    if let Some(credit_limit) = args.credit_limit {
        card.credit_limit = credit_limit;
    }
    if let Some(billing_day) = args.billing_day {
        card.billing_day = billing_day.as_i16();
    }
    if let Some(due_day) = args.due_day {
        card.due_day = due_day.as_i16();
    }
    if let Some(inactive) = args.inactive {
        card.active = !inactive;
    }

    let updated = card.update(&ctx.db.pool).await?;
    crate::handlers::render_success(ctx, &updated);
    Ok(())
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<(), CreditCardError> {
    CreditCard::delete(&ctx.db.pool, id).await?;
    crate::handlers::render_success(ctx, &serde_json::json!({ "deleted": true, "id": id }));
    Ok(())
}
