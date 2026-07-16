use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum InstallmentCmd {
    /// Adds a new installment
    Add(AddInstallmentArgs),

    /// Lists the installments
    List {
        #[arg(long, short)]
        limit: Option<usize>,
    },

    /// Shows details of an installment and its invoices
    Show {
        #[arg(short, long)]
        id: i32,
    },

    /// Adjusts the cents of a specific invoice in the installment
    Adjust(AdjustInstallmentArgs),

    /// Deletes an installment (reverts associated invoices if possible)
    Delete {
        #[arg(short, long)]
        id: i32,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum InstallmentError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Model error: {0}")]
    Model(#[from] crate::models::ModelError),
    #[error("Parse error: {0}")]
    Parse(String),
}

impl TryFrom<AddInstallmentArgs> for crate::models::installment::NewInstallment {
    type Error = InstallmentError;

    fn try_from(args: AddInstallmentArgs) -> Result<Self, Self::Error> {
        use std::str::FromStr;
        Ok(Self {
            credit_card_id: args.card_id,
            category_id: args.category_id,
            description: crate::models::types::NonEmptyString::from_str(&args.description)
                .map_err(|e| InstallmentError::Parse(e.to_string()))?,
            total_amount: crate::models::types::PositiveAmount::from_str(&args.total_amount)
                .map_err(|e| InstallmentError::Parse(e.to_string()))?,
            installments_count: args.installments_count,
            date: match args.date {
                Some(d) => chrono::NaiveDate::from_str(&d)
                    .map_err(|e| InstallmentError::Parse(e.to_string()))?,
                None => chrono::Utc::now().naive_local().date(),
            },
        })
    }
}

impl InstallmentCmd {
    pub async fn handle(self, ctx: &crate::context::AppContext) -> Result<(), InstallmentError> {
        match self {
            Self::Add(args) => {
                let res = crate::handlers::installment::add(ctx, args).await?;
                crate::commands::render_success(ctx, &res);
                Ok(())
            }
            Self::List { limit } => {
                let res = crate::handlers::installment::list(ctx, limit).await?;
                crate::commands::render_success(ctx, &res);
                Ok(())
            }
            Self::Show { id } => {
                let res = crate::handlers::installment::show(ctx, id).await?;
                // Different handle for this because is a more complex structure
                if ctx.json_output {
                    crate::commands::render_success(ctx, &res);
                } else {
                    println!("{:#?}", res.installment);
                    println!("Transações:");
                    for tx in res.transactions {
                        println!(
                            "- Parcela {}: {} ({})",
                            tx.installment_number.as_ref().map(|n| n.to_string()).unwrap_or_else(|| "?".to_string()),
                            tx.amount.as_decimal(),
                            tx.date
                        );
                    }
                }
                Ok(())
            }
            Self::Adjust(args) => {
                let res = crate::handlers::installment::adjust(ctx, args).await?;
                crate::commands::render_success(ctx, &res);
                Ok(())
            }
            Self::Delete { id } => {
                let res = crate::handlers::installment::delete(ctx, id).await?;
                crate::commands::render_success(ctx, &res);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Args)]
pub struct AddInstallmentArgs {
    /// Credit card ID
    #[arg(long)]
    pub card_id: i32,

    /// Category ID (optional)
    #[arg(long)]
    pub category_id: Option<i32>,

    /// Purchase description
    #[arg(short, long)]
    pub description: String,

    /// Total amount of the installment purchase
    #[arg(short, long)]
    pub total_amount: String,

    /// Number of installments
    #[arg(short, long)]
    pub installments_count: crate::models::types::InstallmentCount,

    /// Purchase date (YYYY-MM-DD format). Default: today
    #[arg(long)]
    pub date: Option<String>,
}

#[derive(Debug, Args)]
pub struct AdjustInstallmentArgs {
    /// Base installment ID
    pub id: i32,

    /// Installment number to be adjusted (e.g., 1, 2, 3)
    #[arg(short, long)]
    pub number: crate::models::types::InstallmentNumber,

    /// New exact value of the installment (e.g., 33.34)
    #[arg(short, long)]
    pub amount: String,
}
