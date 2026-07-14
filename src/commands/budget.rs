use crate::models::types::{BudgetPeriod, PositiveAmount};
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum BudgetCmd {
    Add {
        #[arg(long)]
        category_id: Option<i32>,

        #[arg(long)]
        tag_id: Option<i32>,

        #[arg(short, long)]
        limit: PositiveAmount,

        #[arg(short, long)]
        period: BudgetPeriod,
    },
    List {
        #[arg(short, long)]
        date: Option<chrono::NaiveDate>,
    },
    Delete {
        #[arg(short, long)]
        id: i32,
    },
}

impl BudgetCmd {
    pub async fn handle(
        self,
        ctx: &crate::context::AppContext,
    ) -> Result<(), crate::handlers::budget::BudgetError> {
        match self {
            BudgetCmd::Add {
                category_id,
                tag_id,
                limit,
                period,
            } => {
                let res =
                    crate::handlers::budget::add(ctx, category_id, tag_id, limit, period).await?;
                crate::commands::render_success(ctx, &res);
                Ok(())
            }
            BudgetCmd::List { date } => {
                let res = crate::handlers::budget::list(ctx, date).await?;
                crate::commands::render_success(ctx, &res);
                Ok(())
            }
            BudgetCmd::Delete { id } => {
                let res = crate::handlers::budget::delete(ctx, id).await?;
                crate::commands::render_success(ctx, &res);
                Ok(())
            }
        }
    }
}
