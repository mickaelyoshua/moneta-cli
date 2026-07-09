use clap::Parser;
use moneta_cli::{
    cli::{Cli, Command},
    config::Config,
    context::AppContext,
    db::Db,
};

async fn run() -> Result<(), moneta_cli::error::AppError> {
    moneta_cli::telemetry::init_telemetry();

    let cli = Cli::parse();

    let config = Config::load(cli.config)?;
    let db = Db::new(&config.database_url, config.max_connections).await?;
    let ctx = AppContext::new(db, cli.json);

    let command_name = match cli.command {
        Command::Transaction { .. } => "Transaction",
        Command::Category { .. } => "Category",
        Command::Account { .. } => "Account",
        Command::CreditCard { .. } => "CreditCard",
        Command::Invoice { .. } => "Invoice",
        Command::Installment { .. } => "Installment",
        Command::Budget { .. } => "Budget",
        Command::Recurrence { .. } => "Recurrence",
        Command::Overview(_) => "Overview",
    };

    tracing::info!("Starting command flow: {}", command_name);

    match cli.command {
        Command::Transaction { action } => action.handle(&ctx).await?,
        Command::Category { action } => action.handle(&ctx).await?,
        Command::Account { action } => action.handle(&ctx).await?,
        Command::CreditCard { action } => action.handle(&ctx).await?,
        Command::Invoice { action } => action.handle(&ctx).await?,
        Command::Installment { action } => action.handle(&ctx).await?,
        Command::Budget { action } => action.handle(&ctx).await?,
        Command::Recurrence { action } => action.handle(&ctx).await?,
        Command::Overview(cmd) => cmd.handle(&ctx).await?,
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        tracing::error!("Fatal error: {:#}", e);
        std::process::exit(1);
    }
}
