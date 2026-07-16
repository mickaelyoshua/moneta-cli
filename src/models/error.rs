use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Not found")]
    NotFound,
    #[error("Business logic error: {0}")]
    BusinessLogic(String),
}
