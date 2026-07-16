use clap::Subcommand;
use crate::models::types::{Month, Year};

#[derive(thiserror::Error, Debug)]
pub enum InvoiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Subcommand)]
pub enum InvoiceCmd {
    View {
        #[arg(short, long)]
        credit_card_id: i32,
        #[arg(short, long)]
        limit: Option<usize>,
    },
    Close {
        #[arg(short, long)]
        credit_card_id: i32,
        #[arg(short, long)]
        month: Month,
        #[arg(short, long)]
        year: Year,
    },
    Reopen {
        #[arg(short, long)]
        credit_card_id: i32,
        #[arg(short, long)]
        month: Month,
        #[arg(short, long)]
        year: Year,
    },
    Pay {
        #[arg(short, long)]
        credit_card_id: i32,
        #[arg(short, long)]
        month: Month,
        #[arg(short, long)]
        year: Year,
        #[arg(short, long)]
        account_id: i32,
    },
}

impl InvoiceCmd {
    pub async fn handle(
        self,
        ctx: &crate::context::AppContext,
    ) -> Result<(), crate::error::AppError> {
        match self {
            Self::View {
                credit_card_id,
                limit,
            } => {
                let res = crate::handlers::invoice::view(ctx, credit_card_id, limit).await?;
                crate::commands::render_success(ctx, &res);
            }
            Self::Close {
                credit_card_id,
                month,
                year,
            } => {
                let res = crate::handlers::invoice::close(ctx, credit_card_id, month, year).await?;
                crate::commands::render_success(ctx, &res);
            }
            Self::Reopen {
                credit_card_id,
                month,
                year,
            } => {
                let res =
                    crate::handlers::invoice::reopen(ctx, credit_card_id, month, year).await?;
                crate::commands::render_success(ctx, &res);
            }
            Self::Pay {
                credit_card_id,
                month,
                year,
                account_id,
            } => {
                let res =
                    crate::handlers::invoice::pay(ctx, credit_card_id, month, year, account_id)
                        .await?;
                crate::commands::render_success(ctx, &res);
            }
        }
        Ok(())
    }
}
