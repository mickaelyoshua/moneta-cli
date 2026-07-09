use clap::{Parser, Subcommand};

use crate::commands::{
    account::AccountCmd, category::CategoryCmd, credit_card::CreditCardCmd,
    transaction::TransactionCmd, invoice::InvoiceCmd, installment::InstallmentCmd,
};

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
    Category {
        #[command(subcommand)]
        action: CategoryCmd,
    },
    Account {
        #[command(subcommand)]
        action: AccountCmd,
    },
    CreditCard {
        #[command(subcommand)]
        action: CreditCardCmd,
    },
    Invoice {
        #[command(subcommand)]
        action: InvoiceCmd,
    },
    Installment {
        #[command(subcommand)]
        action: InstallmentCmd,
    },
}
