//! Business Logic Handlers
//!
//! This module contains the core use cases of the application. Handlers coordinate
//! data flow between the CLI layer and the data models. They are decoupled from
//! CLI-specific concerns (like JSON output formatting) and return domain errors.

pub mod account;
pub mod budget;
pub mod category;
pub mod credit_card;
pub mod installment;
pub mod invoice;
pub mod overview;
pub mod recurrence;
pub mod transaction;
pub mod import;
