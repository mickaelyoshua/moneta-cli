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

pub async fn balance(ctx: &AppContext, account_id: i32) -> Result<(), AccountError> {
    let balance = Account::balance(&ctx.db.pool, account_id).await?;

    crate::handlers::render_success(ctx, &balance);

    Ok(())
}

pub async fn update(
    ctx: &AppContext,
    args: crate::commands::account::UpdateAccountArgs,
) -> Result<(), AccountError> {
    let mut acc = Account::find_by_id(&ctx.db.pool, args.id).await?;

    if let Some(name) = args.name {
        acc.name = name;
    }
    if let Some(account_type) = args.account_type {
        acc.account_type = account_type;
    }
    if let Some(no_debit_card) = args.no_debit_card {
        acc.has_debit_card = !no_debit_card;
    }
    if let Some(inactive) = args.inactive {
        acc.active = !inactive;
    }

    let updated = acc.update(&ctx.db.pool).await?;
    crate::handlers::render_success(ctx, &updated);
    Ok(())
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<(), AccountError> {
    Account::delete(&ctx.db.pool, id).await?;
    crate::handlers::render_success(ctx, &serde_json::json!({ "deleted": true, "id": id }));
    Ok(())
}
