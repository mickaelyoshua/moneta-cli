# AI Categorization & Import Rules

Moneta CLI AI memory.
AI checks rules here before asking user confirmation. AI guesses Category/Tags.

## CSV Import Guidelines (Para a IA)

Quando o usuário pedir para converter uma fatura/extrato (PDF/imagem) para CSV, gere o CSV seguindo **exatamente** este formato e regras:

### Formato do CSV (Header Obrigatório)

`date,amount,description,category,tags,installments,source_type,source_name`

- `date`: YYYY-MM-DD
- `amount`: Valor numérico positivo (ex: 15.50).
- `description`: Nome do estabelecimento/transação limpo e padronizado.
- `category`: Categoria sugerida (deve existir no banco).
- `tags`: Separadas por pipe `|` (ex: `comida|ifood`).
- `installments`: Se for parcelado, preencher o número total de parcelas e a parcela atual (ex: `3/5`). Se não, vazio.
- `source_type`: `account` ou `credit_card`.
- `source_name`: Nome da conta ou cartão (ex: `Nubank`, `Inter`).

### Regras de Comportamento (IA)

1. Analise o extrato e deduza a `category` e `tags` baseando-se no `Vendor Mapping` abaixo.
2. Se a categoria não for óbvia, sugira uma nova (o CLI lidará com a criação se for necessário, ou peça revisão ao usuário).
3. Padronize os nomes (`description`). "UBER DO BRASIL \*TRIP" -> "Uber".
4. Atualize ativamente o `Vendor Mapping` neste arquivo quando o usuário aprovar uma categorização nova.

## Vendor Mapping

| Vendor / Sender                 | Category    | Tags       | Notes                  |
| :------------------------------ | :---------- | :--------- | :--------------------- |
| Ivaldo Torres (Banco do Brasil) | Alimentação | `comida`   |                        |
| Uber                            | Transporte  | `app`      | Check if not Uber Eats |
| iFood                           | Alimentação | `delivery` |                        |

## Update Instructions

- Update table when recognizing a confirmed pattern.
- `Category` must be valid in DB.
- `Tags` can be flexible.
