# Moneta CLI

CLI em Rust para gestão de finanças pessoais via PostgreSQL.
Projetado tanto para uso humano rápido quanto como backend/engine para Agentes de IA que processam recibos e faturas.

## Funcionalidades

- **Contas e Cartões:** Gestão de múltiplas contas e cartões de crédito.
- **Transações Unificadas:** Despesas, receitas e transferências (`transaction`).
- **Agrupamento:** Parcelamentos (`installment`), Faturas (`invoice`), e Transações Recorrentes (`recurrence`).
- **Classificação:** Categorias estritas (`category`) e tags flexíveis (`tag`).
- **Orçamentos:** Limites de gastos por categoria/tag (`budget`).
- **AI-Ready:** Suporte nativo a output `--json` para ser consumido por IAs externas.

## Pré-requisitos

- [Rust](https://rustup.rs/) (cargo)
- PostgreSQL
- [sqlx-cli](https://crates.io/crates/sqlx-cli) (`cargo install sqlx-cli`)

## Setup Local

1. Suba o banco de dados via Docker:
   ```bash
   docker-compose up -d
   ```

2. Crie e rode as migrations:
   ```bash
   sqlx database create
   sqlx migrate run
   ```

3. (Opcional) Copie as configurações:
   ```bash
   cp config.example.toml config.toml
   ```

## Compilação e Uso

Para buildar em release:
```bash
cargo build --release
```

Para rodar comandos via cargo (desenvolvimento):
```bash
cargo run -- <COMMAND> [OPTIONS]
```

### Exemplos Humanos

**Criar uma Categoria:**
```bash
cargo run -- category create --name "Alimentação" --type "expense"
```

**Criar uma Conta:**
```bash
cargo run -- account create --name "Nubank"
```

### Exemplos para IA

Agentes de IA devem utilizar a flag global `--json` para receber output estruturado.

```bash
cargo run -- --json transaction create \
    --amount 45.00 \
    --date "2026-07-09" \
    --description "Ifood" \
    --category-id 1 \
    --account-id 1 \
    --type expense
```

## Arquitetura e IA

- Para detalhes da arquitetura, veja [architecture.md](architecture.md).
- Para como IAs devem interagir com a CLI, veja [AGENTS.md](AGENTS.md).
