#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),
    #[error("Transaction error: {0}")]
    Transaction(#[from] crate::commands::transaction::TransactionError),
    #[error("Category error: {0}")]
    Category(#[from] crate::commands::category::CategoryError),
    #[error("Account error: {0}")]
    Account(#[from] crate::commands::account::AccountError),
    #[error("Credit Card error: {0}")]
    CreditCard(#[from] crate::commands::credit_card::CreditCardError),
}
