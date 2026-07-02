# Regras de Categorização IA

Este arquivo atua como memória viva para a IA da Moneta CLI.
Quando a IA parsear extratos ou notas fiscais, ela deve checar as regras abaixo antes de solicitar confirmação ao usuário.
O usuário confirmará todas as entradas, mas a IA usará este arquivo para adivinhar a `Categoria` e as `Tags` corretas de forma determinística.

## Mapeamento de Fornecedores

| Fornecedor / Remetente | Categoria | Tags Recomendadas | Notas |
| :--- | :--- | :--- | :--- |
| Ivaldo Torres (Banco do Brasil) | Alimentação | `comida` | |
| Uber | Transporte | `app` | Checar se não foi Uber Eats (Alimentação) |
| iFood | Alimentação | `delivery` | |

## Instruções para Atualizar
- Ao notar um padrão confirmado pelo usuário, a IA deve atualizar esta tabela proativamente.
- A coluna `Categoria` deve apontar obrigatoriamente para um nome válido na tabela `categories` do DB.
- A coluna `Tags Recomendadas` pode conter qualquer termo flexível.
