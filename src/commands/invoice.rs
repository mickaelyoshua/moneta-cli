use clap::Subcommand;

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
        month: i16,
        #[arg(short, long)]
        year: i16,
    },
    Reopen {
        #[arg(short, long)]
        credit_card_id: i32,
        #[arg(short, long)]
        month: i16,
        #[arg(short, long)]
        year: i16,
    },
    Pay {
        #[arg(short, long)]
        credit_card_id: i32,
        #[arg(short, long)]
        month: i16,
        #[arg(short, long)]
        year: i16,
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
            } => crate::handlers::invoice::view(ctx, credit_card_id, limit).await?,
            Self::Close {
                credit_card_id,
                month,
                year,
            } => crate::handlers::invoice::close(ctx, credit_card_id, month, year).await?,
            Self::Reopen {
                credit_card_id,
                month,
                year,
            } => crate::handlers::invoice::reopen(ctx, credit_card_id, month, year).await?,
            Self::Pay {
                credit_card_id,
                month,
                year,
                account_id,
            } => {
                crate::handlers::invoice::pay(ctx, credit_card_id, month, year, account_id).await?
            }
        }
        Ok(())
    }
}
