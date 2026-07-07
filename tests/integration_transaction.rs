use moneta_cli::models::transaction::{NewTransaction, Transaction};
use moneta_cli::models::types::{
    NonEmptyString, PositiveAmount, TransactionSource, TransactionType,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::str::FromStr;

#[sqlx::test]
async fn test_transaction_insert_and_find_all(pool: PgPool) {
    // Setup - Injecting dependencies via dynamic SQL to avoid requiring compile-time sqlx prepare
    let ctg_id: i32 = sqlx::query(
        "INSERT INTO categories (name, category_type) VALUES ('Food', 'expense') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("setup category")
    .get("id");

    let account_id: i32 = sqlx::query(
        "INSERT INTO accounts (name, account_type) VALUES ('Nubank', 'checking') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("setup account")
    .get("id");

    let new_tx = NewTransaction {
        category_id: ctg_id,
        source: TransactionSource::Account { account_id },
        transaction_type: TransactionType::Expense,
        amount: PositiveAmount::from_str("15.50").unwrap(),
        date: chrono::NaiveDate::from_ymd_opt(2026, 7, 7).unwrap(),
        description: NonEmptyString::from_str("Lunch").unwrap(),
    };

    // Action
    let tx = Transaction::insert(&pool, new_tx)
        .await
        .expect("Failed to insert transaction");

    // Assertion - Verify mapping to database shape
    assert_eq!(tx.category_id, ctg_id);
    assert_eq!(tx.source, TransactionSource::Account { account_id });
    assert_eq!(tx.transaction_type, TransactionType::Expense);
    assert_eq!(tx.amount.as_decimal(), Decimal::new(1550, 2));
    assert_eq!(tx.description.as_str(), "Lunch");
    assert_eq!(
        tx.date,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 7).unwrap()
    );

    // Action 2
    let all = Transaction::find_all(&pool, None)
        .await
        .expect("Failed to find_all");

    // Assertion 2
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, tx.id);
}
