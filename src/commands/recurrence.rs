use crate::models::types::{NonEmptyString, PositiveAmount, RecurrenceFrequency, TransactionType};
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum RecurrenceCmd {
    Add {
        #[arg(short, long)]
        category_id: i32,

        #[arg(long, group = "source")]
        account_id: Option<i32>,

        #[arg(long, group = "source")]
        credit_card_id: Option<i32>,

        #[arg(short, long)]
        transaction_type: TransactionType,

        #[arg(short, long)]
        amount: PositiveAmount,

        #[arg(short, long)]
        description: NonEmptyString,

        #[arg(short, long)]
        frequency: RecurrenceFrequency,

        #[arg(short, long)]
        start_date: chrono::NaiveDate,

        #[arg(short, long)]
        end_date: Option<chrono::NaiveDate>,
    },
    List,
    Update {
        #[arg(short, long)]
        id: i32,

        #[arg(short, long)]
        category_id: Option<i32>,

        #[arg(long, group = "source")]
        account_id: Option<i32>,

        #[arg(long, group = "source")]
        credit_card_id: Option<i32>,

        #[arg(short, long)]
        transaction_type: Option<TransactionType>,

        #[arg(short, long)]
        amount: Option<PositiveAmount>,

        #[arg(short, long)]
        description: Option<NonEmptyString>,

        #[arg(short, long)]
        frequency: Option<RecurrenceFrequency>,

        #[arg(short, long)]
        start_date: Option<chrono::NaiveDate>,
    },
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
            RecurrenceCmd::Add {
                category_id,
                account_id,
                credit_card_id,
                transaction_type,
                amount,
                description,
                frequency,
                start_date,
                end_date,
            } => {
                crate::handlers::recurrence::add(
                    ctx,
                    crate::models::recurrence::NewRecurrence {
                        category_id,
                        source: if let Some(id) = account_id {
                            crate::models::types::TransactionSource::Account { account_id: id }
                        } else {
                            crate::models::types::TransactionSource::CreditCard {
                                credit_card_id: credit_card_id.expect("Requires account or card"),
                            }
                        },
                        transaction_type,
                        amount,
                        description,
                        frequency,
                        start_date,
                        end_date,
                    },
                )
                .await
            }
            RecurrenceCmd::List => crate::handlers::recurrence::list(ctx).await,
            RecurrenceCmd::Update {
                id,
                category_id,
                account_id,
                credit_card_id,
                transaction_type,
                amount,
                description,
                frequency,
                start_date,
            } => {
                crate::handlers::recurrence::update(
                    ctx,
                    id,
                    crate::models::recurrence::UpdateRecurrencePayload {
                        category_id,
                        account_id,
                        credit_card_id,
                        transaction_type,
                        amount,
                        description,
                        frequency,
                        start_date,
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
