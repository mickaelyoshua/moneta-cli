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

pub async fn update(
    ctx: &AppContext,
    args: crate::commands::category::UpdateCategoryArgs,
) -> Result<(), CategoryError> {
    let mut ctg = crate::models::category::Category::find_by_id(&ctx.db.pool, args.id).await?;

    if let Some(name) = args.name {
        ctg.name = name;
    }
    if let Some(category_type) = args.category_type {
        ctg.category_type = category_type;
    }
    if let Some(inactive) = args.inactive {
        ctg.active = !inactive;
    }

    let updated = ctg.update(&ctx.db.pool).await?;
    crate::handlers::render_success(ctx, &updated);
    Ok(())
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<(), CategoryError> {
    crate::models::category::Category::delete(&ctx.db.pool, id).await?;
    crate::handlers::render_success(ctx, &serde_json::json!({ "deleted": true, "id": id }));
    Ok(())
}
