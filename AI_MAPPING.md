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
