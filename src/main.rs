use clap::Parser;
use moneta_cli::{
    cli::{Cli, Command},
    config::Config,
    context::AppContext,
    db::Db,
};

async fn run() -> Result<(), moneta_cli::error::AppError> {
    let cli = Cli::parse();

    let config = Config::load(cli.config)?;
    let db = Db::new(&config.database_url, config.max_connections).await?;
    let ctx = AppContext::new(db, cli.json);

    match cli.command {
        Command::Transaction { action } => {
            action.handle(&ctx).await?;
        }
        Command::Category { action } => {
            action.handle(&ctx).await?;
        }
        Command::Account { action } => {
            action.handle(&ctx).await?;
        }
        Command::CreditCard { action } => {
            action.handle(&ctx).await?;
        }
        Command::Invoice { action } => {
            action.handle(&ctx).await?;
        }
        Command::Installment { action } => {
            action.handle(&ctx).await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Fatal error: {:#}", e);
        std::process::exit(1);
    }
}
