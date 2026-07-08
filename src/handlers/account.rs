use crate::{
    commands::account::{AccountError, AddAccountArgs},
    context::AppContext,
    models::account::{Account, NewAccount},
};

pub async fn add(ctx: &AppContext, args: AddAccountArgs) -> Result<(), AccountError> {
    let new_acc: NewAccount = args.try_into()?;
    let acc = Account::insert(&ctx.db.pool, new_acc).await?;

    crate::handlers::render_success(ctx, &acc);

    Ok(())
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<(), AccountError> {
    let accounts = Account::find_all(&ctx.db.pool, limit).await?;

    crate::handlers::render_success(ctx, &accounts);

    Ok(())
}
