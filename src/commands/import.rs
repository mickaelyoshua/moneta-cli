use crate::context::AppContext;
use clap::Parser;

#[derive(thiserror::Error, Debug)]
pub enum ImportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Model error: {0}")]
    Model(#[from] crate::models::ModelError),
    #[error("Handler error: {0}")]
    Handler(String),
}

#[derive(Debug, Parser)]
pub struct ImportCommand {
    #[arg(help = "Caminho para o arquivo CSV de transações a ser importado")]
    pub file: std::path::PathBuf,

    #[arg(
        long,
        help = "Apenas exibe as ações a serem feitas, sem alterar o banco de dados"
    )]
    pub dry_run: bool,
}

impl ImportCommand {
    pub async fn handle(self, ctx: &AppContext) -> Result<(), crate::error::AppError> {
        let res = crate::handlers::import::process_csv(ctx, self).await?;
        crate::commands::render_success(ctx, &res);
        Ok(())
    }
}
