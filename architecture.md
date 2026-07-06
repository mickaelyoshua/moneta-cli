# Arquitetura Moneta CLI (C4 Model)

Este documento descreve a arquitetura do projeto utilizando diagramas estruturados no formato C4 Model via Mermaid.

## Nível 1: Contexto do Sistema (System Context)

Mostra o panorama geral: o usuário, o sistema Moneta e agentes externos.

```mermaid
C4Context
  title Moneta CLI - System Context (Nível 1)

  Person(user, "Usuário Finaçneiro", "Pessoa que deseja registrar e organizar finanças pessoais")
  System(moneta, "Moneta CLI", "App de linha de comando (Rust) para gerenciamento financeiro unificado")
  SystemDb(postgres, "PostgreSQL", "Banco de dados central de registros")
  System_Ext(ai_agent, "Agente IA (Futuro)", "Lê PDFs/CSVs e alimenta a CLI via comandos estruturados")

  Rel(user, moneta, "Executa comandos no terminal", "CLI")
  Rel(ai_agent, moneta, "Automatiza inserção de dados", "CLI/JSON")
  Rel(moneta, postgres, "Armazena transações, categorias e tags", "TCP/SQL")
```

## Nível 2: Containers

Aplica um zoom no sistema "Moneta CLI" para mostrar os blocos de construção técnicos.

```mermaid
C4Container
  title Moneta CLI - Containers (Nível 2)

  Person(user, "Usuário / Agente IA", "Via Terminal")
  
  System_Boundary(moneta_sys, "Sistema Moneta") {
    Container(cli, "Interface CLI", "Rust / clap + serde", "Recebe comandos, valida argumentos, formata saídas (texto/JSON)")
    Container(core, "Regras de Domínio", "Rust", "Processa transações unificadas, faturas (installments) e recorrências")
    Container(db_layer, "Camada de Dados", "Rust / sqlx / tokio", "Executa queries assíncronas no banco")
  }
  
  SystemDb(postgres, "Banco de Dados", "PostgreSQL", "Tabela unificada 'transactions', 'categories', 'tags'")

  Rel(user, cli, "Dispara comandos")
  Rel(cli, core, "Despacha rotas de comando")
  Rel(core, db_layer, "Solicita persistência")
  Rel(db_layer, postgres, "Lê/Escreve via conexão async", "SQL/TCP")
```
