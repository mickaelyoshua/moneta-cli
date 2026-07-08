use chrono::NaiveDate;
use moneta_cli::models::account::{Account, NewAccount};
use moneta_cli::models::category::{Category, NewCategory};
use moneta_cli::models::credit_card::{CreditCard, NewCreditCard};
use moneta_cli::models::invoice::Invoice;
use moneta_cli::models::transaction::{NewTransaction, Transaction};
use moneta_cli::models::types::{
    AccountType, CategoryType, DayOfMonth, NonEmptyString, NonNegativeAmount, PositiveAmount,
    TransactionSource, TransactionType,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_credit_card_transaction_updates_invoice(pool: PgPool) {
    // 1. Arrange: Setup account, card, and category
    let ctg = Category::insert(
        &pool,
        NewCategory {
            name: NonEmptyString::from_str("Eletrônicos").unwrap(),
            category_type: CategoryType::Expense,
            active: true,
        },
    )
    .await
    .unwrap();

    let acc = Account::insert(
        &pool,
        NewAccount {
            name: NonEmptyString::from_str("Conta").unwrap(),
            account_type: AccountType::Checking,
            has_debit_card: true,
            active: true,
        },
    )
    .await
    .unwrap();

    let card = CreditCard::insert(
        &pool,
        NewCreditCard {
            account_id: acc.id,
            name: NonEmptyString::from_str("Cartão").unwrap(),
            credit_limit: NonNegativeAmount::from_str("5000.00").unwrap(),
            billing_day: DayOfMonth::from_str("25").unwrap(),
            due_day: DayOfMonth::from_str("5").unwrap(),
            active: true,
        },
    )
    .await
    .unwrap();

    // 2. Act: Insert TWO transactions on the same month
    let tx_date = NaiveDate::from_ymd_opt(2026, 7, 20).unwrap();
    
    let tx1 = Transaction::insert(
        &pool,
        NewTransaction {
            category_id: Some(ctg.id),
            source: TransactionSource::CreditCard {
                credit_card_id: card.id,
            },
            transaction_type: TransactionType::Expense,
            amount: PositiveAmount::from_str("100.00").unwrap(),
            date: tx_date,
            description: NonEmptyString::from_str("Mouse").unwrap(),
        },
    )
    .await
    .unwrap();

    let tx2 = Transaction::insert(
        &pool,
        NewTransaction {
            category_id: Some(ctg.id),
            source: TransactionSource::CreditCard {
                credit_card_id: card.id,
            },
            transaction_type: TransactionType::Expense,
            amount: PositiveAmount::from_str("50.00").unwrap(),
            date: tx_date,
            description: NonEmptyString::from_str("Teclado").unwrap(),
        },
    )
    .await
    .unwrap();

    // 3. Assert
    assert!(tx1.invoice_id.is_some(), "Transaction 1 deve estar associada a uma fatura");
    assert_eq!(tx1.invoice_id, tx2.invoice_id, "Ambas devem estar na mesma fatura");

    // Verificar se a fatura foi atualizada corretamente (100 + 50 = 150)
    let invoice = sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = $1")
        .bind(tx1.invoice_id.unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(invoice.total_amount, Decimal::from_str("150.00").unwrap(), "Total da fatura deve ser a soma das transações");
}
