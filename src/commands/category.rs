use clap::{self, Subcommand};

use crate::models::{CategoryType, NonEmptyString};

#[derive(thiserror::Error, Debug)]
pub enum CategoryError {
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
        unimplemented!()
    }
}
