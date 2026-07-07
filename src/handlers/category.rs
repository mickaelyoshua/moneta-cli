use crate::{
    commands::category::{AddCategoryArgs, CategoryError},
    context::AppContext,
};

pub async fn add(ctx: &AppContext, args: AddCategoryArgs) -> Result<(), CategoryError> {
    let new_ctg: crate::models::category::NewCategory = args.try_into()?;
    let ctg = crate::models::category::Category::insert(&ctx.db.pool, new_ctg).await?;

    crate::handlers::render_success(ctx, &ctg);

    Ok(())
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<(), CategoryError> {
    let categories = crate::models::category::Category::find_all(&ctx.db.pool, limit).await?;

    crate::handlers::render_success(ctx, &categories);
    Ok(())
}
