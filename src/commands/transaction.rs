use crate::context::AppContext;
use crate::models::types::{NonEmptyString, PositiveAmount, TransactionType};
use clap::Subcommand;

#[derive(thiserror::Error, Debug)]
pub enum TransactionError {
    #[error("Conta ou cartão de crédito deve ser informado")]
    MissingSource,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}
#[derive(Debug, Subcommand)]
pub enum TransactionCmd {
    Add(AddTransactionArgs),
    List {
        #[arg(short, long)]
        limit: Option<usize>,
    },
}

#[derive(Debug, clap::Args)]
#[command(group(
    clap::ArgGroup::new("source")
        .required(true)
        .args(["account_id", "credit_card_id"])
))]
pub struct AddTransactionArgs {
    #[arg(long, group = "source")]
    pub account_id: Option<i32>,

    #[arg(long, group = "source")]
    pub credit_card_id: Option<i32>,

    #[arg(short, long)]
    pub category_id: i32,

    #[arg(short, long, default_value = "expense")]
    pub transaction_type: TransactionType,

    #[arg(short, long)]
    pub amount: PositiveAmount,

    #[arg(short, long)]
    pub date: Option<chrono::NaiveDate>,

    #[arg(long)]
    pub description: NonEmptyString,
}

impl TryFrom<AddTransactionArgs> for crate::models::transaction::NewTransaction {
    type Error = TransactionError;

    fn try_from(args: AddTransactionArgs) -> Result<Self, Self::Error> {
        let date = args
            .date
            .unwrap_or_else(|| chrono::Local::now().naive_local().date());

        let source = if let Some(id) = args.account_id {
            crate::models::types::TransactionSource::Account { account_id: id }
        } else if let Some(id) = args.credit_card_id {
            crate::models::types::TransactionSource::CreditCard { credit_card_id: id }
        } else {
            return Err(TransactionError::MissingSource);
        };

        Ok(Self {
            category_id: args.category_id,
            source,
            transaction_type: args.transaction_type,
            amount: args.amount,
            date,
            description: args.description,
        })
    }
}

impl TransactionCmd {
    pub async fn handle(self, ctx: &AppContext) -> Result<(), crate::error::AppError> {
        match self {
            Self::Add(args) => handlers::add(ctx, args).await?,
            Self::List { limit } => handlers::list(ctx, limit).await?,
        }
        Ok(())
    }
}

pub mod handlers {
    use super::*;

    pub async fn add(ctx: &AppContext, args: AddTransactionArgs) -> Result<(), TransactionError> {
        let new_tx: crate::models::transaction::NewTransaction = args.try_into()?;
        let tx = crate::models::transaction::Transaction::insert(&ctx.db.pool, new_tx).await?;

        if ctx.json_output {
            println!("{}", serde_json::to_string(&tx)?);
        } else {
            println!(
                "Transaction #{} of {} added on {}.",
                tx.id,
                tx.amount.as_decimal(),
                tx.date
            );
        }
        Ok(())
    }

    pub async fn list(ctx: &AppContext, limit: Option<usize>) -> Result<(), TransactionError> {
        let txs = crate::models::transaction::Transaction::find_all(&ctx.db.pool, limit).await?;

        if ctx.json_output {
            println!("{}", serde_json::to_string(&txs)?);
        } else {
            for tx in txs {
                let source_id = match tx.source {
                    crate::models::types::TransactionSource::Account { account_id } => {
                        format!("Acc #{}", account_id)
                    }
                    crate::models::types::TransactionSource::CreditCard { credit_card_id } => {
                        format!("Card #{}", credit_card_id)
                    }
                };
                println!(
                    "[{}] #{} - {} {} ({}) -> {}",
                    tx.date,
                    tx.id,
                    if tx.transaction_type == TransactionType::Income {
                        "+"
                    } else {
                        "-"
                    },
                    tx.amount.as_decimal(),
                    source_id,
                    tx.description.as_str()
                );
            }
        }
        Ok(())
    }
}
