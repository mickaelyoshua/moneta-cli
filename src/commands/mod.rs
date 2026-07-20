//! CLI Commands and Input Parsing
//!
//! This module defines the command-line interface structure using `clap`.
//! It is responsible for parsing user input, translating it into arguments
//! for the business handlers, and formatting the output (e.g. JSON or Debug)
//! back to the user via the `render_success` function.

pub mod account;
pub mod budget;
pub mod category;
pub mod credit_card;
pub mod import;
pub mod installment;
pub mod invoice;
pub mod overview;
pub mod recurrence;
pub mod transaction;

use crate::context::AppContext;
use serde::Serialize;

pub fn render_success<T: Serialize + std::fmt::Debug>(ctx: &AppContext, data: &T) {
    if ctx.json_output {
        match serde_json::to_string(data) {
            Ok(json) => println!("{}", json),
            Err(e) => tracing::error!("Failed to serialize output: {}", e),
        }
    } else {
        println!("{:#?}", data);
    }
}
