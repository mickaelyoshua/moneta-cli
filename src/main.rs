use clap::Parser;
use moneta_cli::{cli::Cli, config::Config, context::AppContext, db::Db};

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = Config::load(cli.config)?;
    let db = Db::new(&config.database_url, config.max_connections).await?;
    let _ctx = AppContext::new(db, cli.json);

    match cli.command {
        _ => todo!("a ser implementado"),
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
