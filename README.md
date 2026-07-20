# Moneta CLI

Rust CLI for personal finance management via PostgreSQL.
Designed for fast human use and as a backend/engine for AI Agents processing receipts and invoices.

## Features

- **Accounts & Cards:** Manage multiple accounts and credit cards (`account`, `credit_card`).
- **Unified Transactions:** Expenses, incomes, and transfers (`transaction`).
- **Grouping:** Installments (`installment`), Invoices (`invoice`), and Recurrent Transactions (`recurrence`).
- **Classification:** Strict categories (`category`) and flexible tags (`tag`).
- **Budgets:** Spending limits by category/tag (`budget`).
- **Import:** Bulk import transactions from CSV (`import`).
- **Overview:** Consolidated financial status report (`overview`).
- **AI-Ready:** Native support for `--json` output to be consumed by external AIs.

## Prerequisites

- [Rust](https://rustup.rs/) (cargo)
- PostgreSQL
- [sqlx-cli](https://crates.io/crates/sqlx-cli) (`cargo install sqlx-cli`)

## Local Setup

1. Spin up the database via Docker:
   ```bash
   docker-compose up -d
   ```

2. Create and run migrations:
   ```bash
   sqlx database create
   sqlx migrate run
   ```

3. (Optional) Copy configuration:
   ```bash
   cp config.example.toml config.toml
   ```

## Build and Usage

Run tests to ensure everything is working:
```bash
cargo test
```

Build release:
```bash
cargo build --release
```

Run via cargo (development):
```bash
cargo run -- <COMMAND> [OPTIONS]
```

### Human Examples

**Create Category:**
```bash
cargo run -- category create --name "Food" --type "expense"
```

**Create Account:**
```bash
cargo run -- account create --name "Nubank"
```

**Import CSV (Dry Run):**
```bash
cargo run -- import --file transactions.csv --dry-run
```

### AI Examples

AI agents must use the global `--json` flag to receive structured output.

```bash
cargo run -- --json transaction create \
    --amount 45.00 \
    --date "2026-07-09" \
    --description "Ifood" \
    --category-id 1 \
    --account-id 1 \
    --type expense
```

## Architecture and AI

- For architecture details, see [architecture.md](architecture.md).
- For AI interaction guidelines, see [AGENTS.md](AGENTS.md).
