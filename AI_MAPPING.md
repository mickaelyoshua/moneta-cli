# AI Categorization Rules

Moneta CLI AI memory.
AI checks rules here before asking user confirmation. AI guesses Category/Tags.

## Vendor Mapping

| Vendor / Sender | Category | Tags | Notes |
| :--- | :--- | :--- | :--- |
| Ivaldo Torres (Banco do Brasil) | Alimentação | `comida` | |
| Uber | Transporte | `app` | Check if not Uber Eats |
| iFood | Alimentação | `delivery` | |

## Update Instructions
- Update table when recognizing a confirmed pattern.
- `Category` must be valid in DB.
- `Tags` can be flexible.
