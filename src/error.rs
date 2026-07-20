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
    #[error("Invoice error: {0}")]
    Invoice(#[from] crate::commands::invoice::InvoiceError),
    #[error("Installment error: {0}")]
    Installment(#[from] crate::commands::installment::InstallmentError),
    #[error("Budget error: {0}")]
    Budget(#[from] crate::handlers::budget::BudgetError),
    #[error("Recurrence error: {0}")]
    Recurrence(#[from] crate::handlers::recurrence::RecurrenceError),
    #[error("Model error: {0}")]
    Model(#[from] crate::models::ModelError),
    #[error("Import error: {0}")]
    Import(#[from] crate::commands::import::ImportError),
}
