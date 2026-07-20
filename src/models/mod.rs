//! Domain Models and Types
//!
//! This module contains all the core business entities, their persistence logic, 
//! and the custom types used to enforce the "Parse, don't validate" design pattern.
//! By using strong typing at the boundary, we ensure invalid states are unrepresentable.

pub mod account;
pub mod budget;
pub mod category;
pub mod credit_card;
pub mod error;
pub mod import;
pub mod installment;
pub mod invoice;
pub mod overview;
pub mod recurrence;
pub mod tag;
pub mod transaction;
pub mod types;

pub use error::ModelError;

pub use account::Account;
pub use category::Category;
pub use credit_card::CreditCard;
pub use installment::Installment;
pub use recurrence::Recurrence;
pub use tag::{Tag, TransactionTag};
pub use transaction::Transaction;
pub use types::*;
