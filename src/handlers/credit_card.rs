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
