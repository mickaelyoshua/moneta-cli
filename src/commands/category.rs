use clap::{self, Subcommand};

use crate::models::{CategoryType, NonEmptyString};

#[derive(thiserror::Error, Debug)]
pub enum CategoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Subcommand)]
pub enum CategoryCmd {
    Add(AddCategoryArgs),
    List {
        #[arg(long, short)]
        limit: Option<usize>,
    },
}

#[derive(Debug, clap::Args)]
pub struct AddCategoryArgs {
    #[arg(short, long)]
    pub name: NonEmptyString,

    #[arg(short, long)]
    pub category_type: CategoryType,

    #[arg(long)]
    pub inactive: bool,
}

impl TryFrom<AddCategoryArgs> for crate::models::category::NewCategory {
    type Error = CategoryError;

    fn try_from(args: AddCategoryArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            name: args.name,
            category_type: args.category_type,
            active: !args.inactive,
        })
    }
}

impl CategoryCmd {
    pub async fn handle(self, ctx: &crate::context::AppContext) -> Result<(), crate::error::AppError> {
        match self {
            Self::Add(args) => crate::handlers::category::add(ctx, args).await?,
            Self::List { limit } => crate::handlers::category::list(ctx, limit).await?,
        }
        Ok(())
    }
}
