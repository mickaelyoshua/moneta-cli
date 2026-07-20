# AI Categorization & Import Rules

Moneta CLI AI memory.
AI checks rules here before asking user confirmation. AI guesses Category/Tags.

## CSV Import Guidelines (For AI)

When the user asks to convert an invoice/statement (PDF/image) to CSV, generate the CSV following this **exact** format and rules:

### CSV Format (Header Required)

`date,amount,description,category,tags,installments,source_type,source_name`

- `date`: YYYY-MM-DD
- `amount`: Positive numeric value (e.g. 15.50).
- `description`: Clean and standardized merchant/transaction name.
- `category`: Suggested category (must exist in DB).
- `tags`: Pipe-separated `|` (e.g. `food|ifood`).
- `installments`: If installment plan, fill total and current (e.g. `3/5`). Otherwise, empty.
- `source_type`: `account` or `credit_card`.
- `source_name`: Name of account or card (e.g. `Nubank`, `Inter`).

### Behavioral Rules (AI)

1. Analyze statement and deduce `category` and `tags` based on the `Vendor Mapping` below.
2. If category is not obvious, suggest a new one (CLI handles creation if needed, or ask user for review).
3. Standardize names (`description`). "UBER DO BRASIL \*TRIP" -> "Uber".
4. Actively update the `Vendor Mapping` in this file when the user approves a new categorization.

## Vendor Mapping

| Vendor / Sender                 | Category  | Tags       | Notes                  |
| :------------------------------ | :-------- | :--------- | :--------------------- |
| Ivaldo Torres (Banco do Brasil) | Food      | `food`     |                        |
| Uber                            | Transport | `app`      | Check if not Uber Eats |
| iFood                           | Food      | `delivery` |                        |

## Update Instructions

- Update table when recognizing a confirmed pattern.
- `Category` must be valid in DB.
- `Tags` can be flexible.

## CLI Usage Guide & Business Logic

### CLI Execution Rules
**Rule #1: NEVER USE `cargo run`**. Always use the compiled binary to interact with the database (e.g., `./target/debug/moneta-cli <COMMAND>`). `cargo run` injects a local dev `DATABASE_URL` (via `.cargo/config.toml`) and will write to the local Docker database instead of production.

### Using the JSON flag
AI agents must ALWAYS pass the `--json` or `-j` flag globally to parse output.
Example: `./target/debug/moneta-cli --json account list`

### 1. Accounts & Credit Cards
- **Account**: Stores available balance. Types: `checking`, `savings`. Must have `name`.
- **Credit Card**: Linked to an Account (`account_id`). Has `credit_limit`, `billing_day` (when statement closes, 1-28), `due_day` (when payment is due).

### 2. Transactions
- The central entity. Types: `income`, `expense`, `transfer`.
- Must belong to either an `Account` or a `CreditCard` (`source_type`, `source_name`).
- Can optionally have a `category_id` (must exist) and multiple `tags`.
- If an expense is on a credit card, it gets batched into an `Invoice`.

### 3. Invoices (Faturas)
- Generated automatically by credit card transactions.
- Status: `open`, `closed`, `paid`.
- Closes on the `billing_day`, paid on the `due_day`.

### 4. Installments (Parcelas)
- A single purchase spread over multiple months.
- E.g., 3/5 means the 3rd installment out of 5 total.
- CLI commands generate future `Transaction`s for each installment automatically.

### 5. Recurrences
- Subscriptions or fixed monthly costs (e.g., Netflix).
- Generates transactions based on a frequency (`daily`, `weekly`, `monthly`, `yearly`).

### 6. Budgets
- Spending limits linked to a Category or Tag.
- Periods: `weekly`, `monthly`, `yearly`.
- Tracked via the `overview` command to compare budget limits versus actual spent.
