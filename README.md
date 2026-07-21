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

## Production Setup

Moneta CLI runs in production using the database configured in `config.toml`.

1. Ensure your configuration file has the production database credentials:

   ```bash
   cp config.example.toml config.toml
   # Edit config.toml to include the production database_url
   ```

2. The application will automatically connect to the configured production database and run embedded migrations on startup.

## Development & Testing Setup

For local development and testing, use the Docker container to isolate the environment and avoid affecting production data.

1. Spin up the local database via Docker:

   ```bash
   docker-compose up -d
   ```

2. Ensure your local `config.toml` or `.env` points to the local Docker database (e.g., `postgres://moneta:101010@localhost:5432/moneta`).

3. Create and run migrations locally:
   ```bash
   sqlx database create
   sqlx migrate run
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

Run via compiled binary:

```bash
./target/debug/moneta-cli <COMMAND> [OPTIONS]
```

### Human Examples

**Create Category:**

```bash
./target/debug/moneta-cli category create --name "Food" --type "expense"
```

**Create Account:**

```bash
./target/debug/moneta-cli account create --name "Nubank"
```

**Import CSV (Dry Run):**

```bash
./target/debug/moneta-cli import --file transactions.csv --dry-run
```

### AI Examples

AI agents must use the global `--json` flag to receive structured output.

```bash
./target/debug/moneta-cli --json transaction create \
    --amount 45.00 \
    --date "2026-07-09" \
    --description "Ifood" \
    --category-id 1 \
    --account-id 1 \
    --type expense
```

## Architecture and AI

- For architecture details, see [architecture.md](architecture.md).
- For AI interaction guidelines, see [AI_MAPPING.md](AI_MAPPING.md).
