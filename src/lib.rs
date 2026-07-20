//! Moneta CLI - AI-friendly personal finances CLI
//!
//! # Architecture
//! This project follows a decoupled, command-based architecture:
//! - `cli`: Maps the command-line interface using `clap`.
//! - `commands`: Defines CLI commands and provides an interface between the user and the domain logic.
//! - `handlers`: Contains the core business logic, decoupled from CLI specifics (e.g. JSON output or clap).
//! - `models`: Defines data structures, domain types, and their persistence logic. Follows "Parse, don't validate".
//! - `db`: Provides a generic PostgreSQL persistence layer using `sqlx`.
//!
//! # Extensibility
//! When adding new features:
//! 1. Define the domain model and migrations (`models` and `migrations/`).
//! 2. Implement business logic (`handlers`).
//! 3. Expose the functionality via CLI commands (`commands` and `cli`).

pub mod cli;
pub mod commands;
pub mod config;
pub mod context;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod telemetry;
