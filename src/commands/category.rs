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
    Update(UpdateCategoryArgs),
    Delete {
        #[arg(short, long)]
        id: i32,
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

#[derive(Debug, clap::Args)]
pub struct UpdateCategoryArgs {
    #[arg(short, long)]
    pub id: i32,

    #[arg(short, long)]
    pub name: Option<NonEmptyString>,

    #[arg(short, long)]
    pub category_type: Option<CategoryType>,

    #[arg(long)]
    pub inactive: Option<bool>,
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
    pub async fn handle(
        self,
        ctx: &crate::context::AppContext,
    ) -> Result<(), crate::error::AppError> {
        match self {
            Self::Add(args) => {
                let res = crate::handlers::category::add(ctx, args).await?;
                crate::commands::render_success(ctx, &res);
            }
            Self::List { limit } => {
                let res = crate::handlers::category::list(ctx, limit).await?;
                crate::commands::render_success(ctx, &res);
            }
            Self::Update(args) => {
                let res = crate::handlers::category::update(ctx, args).await?;
                crate::commands::render_success(ctx, &res);
            }
            Self::Delete { id } => {
                let res = crate::handlers::category::delete(ctx, id).await?;
                crate::commands::render_success(ctx, &res);
            }
        }
        Ok(())
    }
}
