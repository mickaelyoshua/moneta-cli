use crate::{
    commands::category::{AddCategoryArgs, CategoryError},
    context::AppContext,
};

pub async fn add(ctx: &AppContext, args: AddCategoryArgs) -> Result<(), CategoryError> {
    let new_ctg: crate::models::category::NewCategory = args.try_into()?;
    let ctg = crate::models::category::Category::insert(&ctx.db.pool, new_ctg).await?;

    if ctx.json_output {
        println!("{}", serde_json::to_string(&ctg)?);
    } else {
        println!(
            "Category #{} of name {} and type {:?} added.",
            ctg.id, ctg.name.as_str(), ctg.category_type,
        );
    }

    Ok(())
}

pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<(), CategoryError> {
    let categories = crate::models::category::Category::find_all(&ctx.db.pool, limit).await?;

    if ctx.json_output {
        println!("{}", serde_json::to_string(&categories)?);
    } else {
        for ctg in categories {
            let status = if ctg.active { "Active" } else { "Inactive" };
            println!(
                "[{}] #{} - {} ({:?})",
                status,
                ctg.id,
                ctg.name.as_str(),
                ctg.category_type
            );
        }
    }
    Ok(())
}
