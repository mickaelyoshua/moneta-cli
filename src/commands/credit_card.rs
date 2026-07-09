use clap::Subcommand;

use crate::models::{
    credit_card::NewCreditCard,
    types::{DayOfMonth, NonEmptyString, NonNegativeAmount},
};

#[derive(thiserror::Error, Debug)]
pub enum CreditCardError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Subcommand)]
pub enum CreditCardCmd {
    Add(AddCreditCardArgs),
    List {
        #[arg(long, short)]
        limit: Option<usize>,
    },
    Update(UpdateCreditCardArgs),
    Delete {
        #[arg(short, long)]
        id: i32,
    },
}

#[derive(Debug, clap::Args)]
pub struct AddCreditCardArgs {
    #[arg(short, long)]
    pub account_id: i32,

    #[arg(short, long)]
    pub name: NonEmptyString,

    #[arg(short = 'l', long)]
    pub credit_limit: NonNegativeAmount,

    #[arg(short, long)]
    pub billing_day: DayOfMonth,

    #[arg(short, long)]
    pub due_day: DayOfMonth,

    #[arg(long)]
    pub inactive: bool,
}

#[derive(Debug, clap::Args)]
pub struct UpdateCreditCardArgs {
    #[arg(short, long)]
    pub id: i32,

    #[arg(short, long)]
    pub name: Option<NonEmptyString>,

    #[arg(short = 'l', long)]
    pub credit_limit: Option<NonNegativeAmount>,

    #[arg(short, long)]
    pub billing_day: Option<DayOfMonth>,

    #[arg(short, long)]
    pub due_day: Option<DayOfMonth>,

    #[arg(long)]
    pub inactive: Option<bool>,
}

impl TryFrom<AddCreditCardArgs> for NewCreditCard {
    type Error = CreditCardError;

    fn try_from(args: AddCreditCardArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            account_id: args.account_id,
            name: args.name,
            credit_limit: args.credit_limit,
            billing_day: args.billing_day,
            due_day: args.due_day,
            active: !args.inactive,
        })
    }
}

impl CreditCardCmd {
    pub async fn handle(
        self,
        ctx: &crate::context::AppContext,
    ) -> Result<(), crate::error::AppError> {
        match self {
            Self::Add(args) => crate::handlers::credit_card::add(ctx, args).await?,
            Self::List { limit } => crate::handlers::credit_card::list(ctx, limit).await?,
            Self::Update(args) => crate::handlers::credit_card::update(ctx, args).await?,
            Self::Delete { id } => crate::handlers::credit_card::delete(ctx, id).await?,
        }
        Ok(())
    }
}
