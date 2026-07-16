use crate::{context::AppContext, error::AppError, handlers::overview::handle_overview};
use chrono::NaiveDate;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct OverviewCmd {
    #[arg(short, long, help = "Reference date for budgets (YYYY-MM-DD)")]
    pub date: Option<NaiveDate>,
}

impl OverviewCmd {
    pub async fn handle(self, ctx: &AppContext) -> Result<(), AppError> {
        let response = handle_overview(ctx, self.date).await?;
        crate::commands::render_success(ctx, &response);
        Ok(())
    }
}
