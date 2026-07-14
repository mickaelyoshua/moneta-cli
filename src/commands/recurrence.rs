use crate::models::types::{NonEmptyString, PositiveAmount, RecurrenceFrequency, TransactionType};
use clap::Subcommand;

#[derive(Debug, clap::Args)]
#[command(group(
    clap::ArgGroup::new("source")
        .required(true)
        .args(["account_id", "credit_card_id"])
))]
pub struct AddRecurrenceArgs {
    #[arg(short, long)]
    pub category_id: i32,

    #[arg(long, group = "source")]
    pub account_id: Option<i32>,

    #[arg(long, group = "source")]
    pub credit_card_id: Option<i32>,

    #[arg(short, long)]
    pub transaction_type: TransactionType,

    #[arg(short, long)]
    pub amount: PositiveAmount,

    #[arg(short, long)]
    pub description: NonEmptyString,

    #[arg(short, long)]
    pub frequency: RecurrenceFrequency,

    #[arg(short, long)]
    pub start_date: chrono::NaiveDate,

    #[arg(short, long)]
    pub end_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, clap::Args)]
#[command(group(
    clap::ArgGroup::new("source")
        .args(["account_id", "credit_card_id"])
))]
pub struct UpdateRecurrenceArgs {
    #[arg(short, long)]
    pub id: i32,

    #[arg(short, long)]
    pub category_id: Option<i32>,

    #[arg(long, group = "source")]
    pub account_id: Option<i32>,

    #[arg(long, group = "source")]
    pub credit_card_id: Option<i32>,

    #[arg(short, long)]
    pub transaction_type: Option<TransactionType>,

    #[arg(short, long)]
    pub amount: Option<PositiveAmount>,

    #[arg(short, long)]
    pub description: Option<NonEmptyString>,

    #[arg(short, long)]
    pub frequency: Option<RecurrenceFrequency>,

    #[arg(short, long)]
    pub start_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Subcommand)]
pub enum RecurrenceCmd {
    Add(AddRecurrenceArgs),
    List,
    Update(UpdateRecurrenceArgs),
    Delete {
        #[arg(short, long)]
        id: i32,
    },
    Sync {
        #[arg(short, long)]
        date: Option<chrono::NaiveDate>,
    },
}

impl RecurrenceCmd {
    pub async fn handle(
        self,
        ctx: &crate::context::AppContext,
    ) -> Result<(), crate::handlers::recurrence::RecurrenceError> {
        match self {
            RecurrenceCmd::Add(args) => {
                crate::handlers::recurrence::add(
                    ctx,
                    crate::models::recurrence::NewRecurrence {
                        category_id: args.category_id,
                        source: if let Some(id) = args.account_id {
                            crate::models::types::TransactionSource::Account { account_id: id }
                        } else {
                            crate::models::types::TransactionSource::CreditCard {
                                credit_card_id: args
                                    .credit_card_id
                                    .expect("Requires account or card"),
                            }
                        },
                        transaction_type: args.transaction_type,
                        amount: args.amount,
                        description: args.description,
                        frequency: args.frequency,
                        start_date: args.start_date,
                        end_date: args.end_date,
                    },
                )
                .await
            }
            RecurrenceCmd::List => crate::handlers::recurrence::list(ctx).await,
            RecurrenceCmd::Update(args) => {
                crate::handlers::recurrence::update(
                    ctx,
                    args.id,
                    crate::models::recurrence::UpdateRecurrencePayload {
                        category_id: args.category_id,
                        account_id: args.account_id,
                        credit_card_id: args.credit_card_id,
                        transaction_type: args.transaction_type,
                        amount: args.amount,
                        description: args.description,
                        frequency: args.frequency,
                        start_date: args.start_date,
                        end_date: None,
                    },
                )
                .await
            }
            RecurrenceCmd::Delete { id } => crate::handlers::recurrence::delete(ctx, id).await,
            RecurrenceCmd::Sync { date } => crate::handlers::recurrence::sync(ctx, date).await,
        }
    }
}
