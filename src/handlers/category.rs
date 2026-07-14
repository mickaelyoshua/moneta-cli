use crate::{
    commands::category::{AddCategoryArgs, CategoryError},
    context::AppContext,
    models::category::Category,
};

pub async fn add(ctx: &AppContext, args: AddCategoryArgs) -> Result<Category, CategoryError> {
    let new_ctg: crate::models::category::NewCategory = args.try_into()?;
    let ctg = Category::insert(&ctx.db.pool, new_ctg).await?;
    Ok(ctg)
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<Vec<Category>, CategoryError> {
    let categories = Category::find_all(&ctx.db.pool, limit).await?;
    Ok(categories)
}

pub async fn update(
    ctx: &AppContext,
    args: crate::commands::category::UpdateCategoryArgs,
) -> Result<Category, CategoryError> {
    let mut ctg = Category::find_by_id(&ctx.db.pool, args.id).await?;

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
    Ok(updated)
}

pub async fn delete(ctx: &AppContext, id: i32) -> Result<serde_json::Value, CategoryError> {
    Category::delete(&ctx.db.pool, id).await?;
    Ok(serde_json::json!({ "deleted": true, "id": id }))
}
