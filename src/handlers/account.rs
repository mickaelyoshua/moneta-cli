use crate::{
    commands::account::{AccountError, AddAccountArgs, UpdateAccountArgs},
    context::AppContext,
    models::account::{Account, AccountBalance, NewAccount},
};

pub async fn add(ctx: &AppContext, args: AddAccountArgs) -> Result<Account, AccountError> {
    let new_acc: NewAccount = args.try_into()?;
    let acc = Account::insert(&ctx.db.pool, new_acc).await?;
    Ok(acc)
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<Vec<Account>, AccountError> {
    let accounts = Account::find_all(&ctx.db.pool, limit).await?;
    Ok(accounts)
}

pub async fn balance(ctx: &AppContext, account_id: i32) -> Result<AccountBalance, AccountError> {
    let balance = Account::balance(&ctx.db.pool, account_id).await?;
    Ok(balance)
}

pub async fn update(ctx: &AppContext, args: UpdateAccountArgs) -> Result<Account, AccountError> {
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
    Ok(updated)
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<serde_json::Value, AccountError> {
    Account::delete(&ctx.db.pool, id).await?;
    Ok(serde_json::json!({ "deleted": true, "id": id }))
}
