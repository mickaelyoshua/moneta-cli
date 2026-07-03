use clap::{Parser, Subcommand};

use crate::commands::transaction::TransactionCmd;

#[derive(clap::Args, Debug)]
pub struct ConfigArgs {
    #[arg(short, long, env = "DATABASE_URL")]
    pub database_url: Option<String>,

    #[arg(short, long, env = "MAX_CONNECTIONS")]
    pub max_connections: Option<u32>,
}

#[derive(Parser)]
#[command(name = "moneta", about = "AI-friendly personal finances CLI")]
pub struct Cli {
    #[arg(short, long, global = true, help = "Forces JSON output")]
    pub json: bool,

    #[command(flatten)]
    pub config: ConfigArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Transaction {
        #[command(subcommand)]
        action: TransactionCmd,
    },
    Category,
}
