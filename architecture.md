# Moneta CLI Architecture

## Level 1: System Context (C4)
```mermaid
C4Context
  title Level 1: Context

  Person(user, "User", "Manages finances")
  System(moneta, "Moneta CLI", "Core application")
  SystemDb(db, "PostgreSQL", "Central storage")
  System_Ext(ai, "AI Agent", "Parses receipts, feeds CLI")

  Rel(user, moneta, "Runs commands", "Terminal")
  Rel(ai, moneta, "Automates entry", "CLI/JSON")
  Rel(moneta, db, "Reads/Writes", "TCP/SQL")
```

## Level 2: Containers (C4)
```mermaid
C4Container
  title Level 2: Containers

  Person(user, "User/AI", "Terminal")
  
  System_Boundary(moneta, "Moneta System") {
    Container(cli, "CLI Interface", "Rust/Clap", "Args parsing, output formatting")
    Container(core, "Domain Logic", "Rust", "Business rules")
    Container(db_layer, "Data Access", "Rust/sqlx", "Async DB operations")
  }
  
  SystemDb(db, "PostgreSQL", "Database", "Unified tables")

  Rel(user, cli, "Calls")
  Rel(cli, core, "Routes")
  Rel(core, db_layer, "Uses")
  Rel(db_layer, db, "Queries", "SQL")
```

## Level 3: Components (C4)
```mermaid
C4Component
  title Level 3: Components

  Container(cli, "CLI Parser", "Clap", "Entrypoint")
  
  Container_Boundary(core, "Core (Rust)") {
    Component(cmd, "Command Handlers", "Maps CLI to Domain")
    Component(tx, "Transactions", "Unified ledger logic")
    Component(entities, "Entities", "Accounts, Categories, Tags")
  }
  
  Container(db_layer, "Repositories", "sqlx", "DB traits implementations")

  Rel(cli, cmd, "Invokes")
  Rel(cmd, tx, "Uses")
  Rel(cmd, entities, "Uses")
  Rel(tx, db_layer, "Persists")
  Rel(entities, db_layer, "Persists")
```

## Level 4: Domain Classes (UML)
```mermaid
classDiagram
  direction BT
  
  class Transaction
  class Installment
  class Recurrence
  class Category
  class Tag
  
  Installment "1" *-- "N" Transaction : Groups
  Recurrence "1" *-- "N" Transaction : Generates
  Category "1" o-- "N" Transaction : Classifies
  Tag "N" o-- "N" Transaction : Tags
```

## Level 4: Execution Flow (UML)
```mermaid
sequenceDiagram
  title Execution Flow (Create Entity)
  
  actor User
  participant CLI as CLI (Clap)
  participant Core as Domain
  participant Repo as DB (sqlx)
  participant PG as PostgreSQL

  User->>CLI: Run command
  CLI->>Core: Parse & Route
  Core->>Repo: Execute DB operation
  Repo->>PG: SQL Query
  PG-->>Repo: Result
  Repo-->>Core: Mapped Struct
  Core-->>CLI: Format Data
  CLI-->>User: STDOUT (Text/JSON)
```
