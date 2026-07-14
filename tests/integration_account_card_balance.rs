use moneta_cli::models::account::{Account, NewAccount};
use moneta_cli::models::category::{Category, NewCategory};
use moneta_cli::models::credit_card::{CreditCard, NewCreditCard};
use moneta_cli::models::types::{
    AccountType, CategoryType, DayOfMonth, NonEmptyString, NonNegativeAmount,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_balance_ignores_credit_card_expenses(pool: PgPool) {
    // 1. Arrange: Criar Conta e Cartão
    let acc = Account::insert(
        &pool,
        NewAccount {
            name: NonEmptyString::from_str("Conta Corrente").unwrap(),
            account_type: AccountType::Checking,
            has_debit_card: true,
            active: true,
        },
    )
    .await
    .unwrap();

    let ctg = Category::insert(
        &pool,
        NewCategory {
            name: NonEmptyString::from_str("Geral").unwrap(),
            category_type: CategoryType::Income,
            active: true,
        },
    )
    .await
    .unwrap();

    let card = CreditCard::insert(
        &pool,
        NewCreditCard {
            account_id: acc.id,
            name: NonEmptyString::from_str("Cartão Principal").unwrap(),
            credit_limit: NonNegativeAmount::from_str("5000.00").unwrap(),
            billing_day: DayOfMonth::from_str("5").unwrap(),
            due_day: DayOfMonth::from_str("10").unwrap(),
            active: true,
        },
    )
    .await
    .unwrap();

    // 2. Act: Inserir transação de RECEITA na Conta (+1000)
    sqlx::query!(
        r#"
        INSERT INTO transactions (category_id, account_id, transaction_type, amount, date, description, status)
        VALUES ($1, $2, 'income'::transaction_type_enum, $3, CURRENT_DATE, 'Salário', 'cleared'::transaction_status_enum)
        "#,
        ctg.id,
        acc.id,
        Decimal::from_str("1000.00").unwrap()
    ).execute(&pool).await.unwrap();

    // Inserir transação de DESPESA no Cartão de Crédito (-300)
    sqlx::query!(
        r#"
        INSERT INTO transactions (category_id, credit_card_id, transaction_type, amount, date, description, status)
        VALUES ($1, $2, 'expense'::transaction_type_enum, $3, CURRENT_DATE, 'Ifood', 'cleared'::transaction_status_enum)
        "#,
        ctg.id,
        card.id,
        Decimal::from_str("300.00").unwrap()
    ).execute(&pool).await.unwrap();

    // 3. Assert: O saldo da conta deve ser 1000, e NÃO 700. O cartão ainda não foi pago.
    let balance = Account::balance(&pool, acc.id).await.unwrap();
    assert_eq!(
        balance.balance,
        Decimal::from_str("1000.00").unwrap(),
        "Saldo não deve descontar faturas abertas"
    );
}
