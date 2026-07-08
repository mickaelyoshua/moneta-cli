use clap::Subcommand;

use crate::models::{AccountType, NonEmptyString};

#[derive(thiserror::Error, Debug)]
pub enum AccountError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Subcommand)]
pub enum AccountCmd {
    Add(AddAccountArgs),
    List {
        #[arg(long, short)]
        limit: Option<usize>,
    },
    Balance {
        #[arg(short, long)]
        id: i32,
    },
    Update(UpdateAccountArgs),
    Delete {
        #[arg(short, long)]
        id: i32,
    },
}

#[derive(Debug, clap::Args)]
pub struct AddAccountArgs {
    #[arg(short, long)]
    pub name: NonEmptyString,

    #[arg(short, long)]
    pub account_type: AccountType,

    #[arg(long)]
    pub no_debit_card: bool,

    #[arg(long)]
    pub inactive: bool,
}

#[derive(Debug, clap::Args)]
pub struct UpdateAccountArgs {
    #[arg(short, long)]
    pub id: i32,

    #[arg(short, long)]
    pub name: Option<NonEmptyString>,

    #[arg(short, long)]
    pub account_type: Option<AccountType>,

    #[arg(long)]
    pub no_debit_card: Option<bool>,

    #[arg(long)]
    pub inactive: Option<bool>,
}

impl TryFrom<AddAccountArgs> for crate::models::account::NewAccount {
    type Error = AccountError;

    fn try_from(args: AddAccountArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            name: args.name,
            account_type: args.account_type,
            has_debit_card: !args.no_debit_card,
            active: !args.inactive,
        })
    }
}

impl AccountCmd {
    pub async fn handle(
        self,
        ctx: &crate::context::AppContext,
    ) -> Result<(), crate::error::AppError> {
        match self {
            Self::Add(args) => crate::handlers::account::add(ctx, args).await?,
            Self::List { limit } => crate::handlers::account::list(ctx, limit).await?,
            Self::Balance { id } => crate::handlers::account::balance(ctx, id).await?,
            Self::Update(args) => crate::handlers::account::update(ctx, args).await?,
            Self::Delete { id } => crate::handlers::account::delete(ctx, id).await?,
        }
        Ok(())
    }
}
