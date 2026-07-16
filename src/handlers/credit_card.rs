use crate::{
    commands::credit_card::{AddCreditCardArgs, CreditCardError},
    context::AppContext,
    models::credit_card::{CreditCard, NewCreditCard},
};

pub async fn add(ctx: &AppContext, args: AddCreditCardArgs) -> Result<CreditCard, CreditCardError> {
    let new_card: NewCreditCard = args.try_into()?;
    let card = CreditCard::insert(&ctx.db.pool, new_card).await?;

    Ok(card)
}

pub async fn list(
    ctx: &AppContext,
    limit: Option<usize>,
) -> Result<Vec<CreditCard>, CreditCardError> {
    let cards = CreditCard::find_all(&ctx.db.pool, limit).await?;

    Ok(cards)
}

pub async fn update(
    ctx: &AppContext,
    args: crate::commands::credit_card::UpdateCreditCardArgs,
) -> Result<CreditCard, CreditCardError> {
    let mut card = CreditCard::find_by_id(&ctx.db.pool, args.id).await?;

    if let Some(name) = args.name {
        card.name = name;
    }
    if let Some(credit_limit) = args.credit_limit {
        card.credit_limit = credit_limit;
    }
    if let Some(billing_day) = args.billing_day {
        card.billing_day = billing_day;
    }
    if let Some(due_day) = args.due_day {
        card.due_day = due_day;
    }
    if let Some(inactive) = args.inactive {
        card.active = !inactive;
    }

    let updated = card.update(&ctx.db.pool).await?;
    Ok(updated)
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<serde_json::Value, CreditCardError> {
    CreditCard::delete(&ctx.db.pool, id).await?;
    Ok(serde_json::json!({ "deleted": true, "id": id }))
}
