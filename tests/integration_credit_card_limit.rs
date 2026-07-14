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
async fn test_used_limit_and_invoice_totals(pool: PgPool) {
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

    let ctg = Category::insert(
        &pool,
        NewCategory {
            name: NonEmptyString::from_str("Compras").unwrap(),
            category_type: CategoryType::Expense,
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
            credit_limit: NonNegativeAmount::from_str("1000.00").unwrap(),
            billing_day: DayOfMonth::from_str("1").unwrap(),
            due_day: DayOfMonth::from_str("5").unwrap(),
            active: true,
        },
    )
    .await
    .unwrap();

    let mut tx = pool.begin().await.unwrap();

    // 1. Inserir Despesa de 100
    Transaction::insert(
        &mut tx,
        NewTransaction {
            category_id: Some(ctg.id),
            source: TransactionSource::CreditCard {
                credit_card_id: card.id,
            },
            transaction_type: TransactionType::Expense,
            amount: PositiveAmount::from_str("100.00").unwrap(),
            date: chrono::Utc::now().naive_local().date(),
            description: NonEmptyString::from_str("Despesa 1").unwrap(),
            installment_id: None,
            installment_number: None,
            tags: vec![],
        },
    )
    .await
    .unwrap();

    // 2. Inserir Estorno (Income) de 30
    Transaction::insert(
        &mut tx,
        NewTransaction {
            category_id: Some(ctg.id),
            source: TransactionSource::CreditCard {
                credit_card_id: card.id,
            },
            transaction_type: TransactionType::Income, // Refund
            amount: PositiveAmount::from_str("30.00").unwrap(),
            date: chrono::Utc::now().naive_local().date(),
            description: NonEmptyString::from_str("Refund").unwrap(),
            installment_id: None,
            installment_number: None,
            tags: vec![],
        },
    )
    .await
    .unwrap();

    tx.commit().await.unwrap();

    // Verificar `used_limit` (esperado: 70)
    let used_limit = CreditCard::used_limit(&pool, card.id).await.unwrap();
    assert_eq!(used_limit, Decimal::from_str("70.00").unwrap());

    // Fatura foi criada no insert
    let invoices = sqlx::query_as::<_, Invoice>("SELECT * FROM invoices")
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(invoices.len(), 1);
    let invoice = &invoices[0];

    // Verificar total atual da fatura (esperado: 70)
    let current_total = Invoice::current_total(&pool, invoice.id).await.unwrap();
    assert_eq!(current_total, Decimal::from_str("70.00").unwrap());

    // Fechar a fatura e validar se o closing_amount gravou certo
    let closed = Invoice::close(&pool, card.id, invoice.month, invoice.year)
        .await
        .unwrap();
    assert_eq!(
        closed.closing_amount,
        Some(Decimal::from_str("70.00").unwrap())
    );
}
