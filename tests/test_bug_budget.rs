use moneta_cli::models::budget::Budget;
use moneta_cli::models::transaction::{NewTransaction, Transaction};
use moneta_cli::models::types::{
    BudgetPeriod, NonEmptyString, PositiveAmount, TransactionSource, TransactionType,
};
use sqlx::PgPool;
use std::str::FromStr;
use rust_decimal::Decimal;

#[sqlx::test]
async fn test_budget_current_spend_with_refund(pool: PgPool) {
    let cat_id = sqlx::query!("INSERT INTO categories (name, category_type) VALUES ('Lazer', 'expense') RETURNING id")
        .fetch_one(&pool).await.unwrap().id;
    let acc_id = sqlx::query!("INSERT INTO accounts (name, account_type) VALUES ('Conta', 'checking') RETURNING id")
        .fetch_one(&pool).await.unwrap().id;

    let budget = Budget::insert(
        &pool,
        Some(cat_id),
        None,
        PositiveAmount::from_str("500.0").unwrap(),
        BudgetPeriod::Monthly,
    )
    .await
    .unwrap();

    let mut db_tx = pool.begin().await.unwrap();

    // Expense
    Transaction::insert(
        &mut *db_tx,
        NewTransaction {
            category_id: Some(cat_id),
            source: TransactionSource::Account { account_id: acc_id },
            transaction_type: TransactionType::Expense,
            amount: PositiveAmount::from_str("150.0").unwrap(),
            date: chrono::NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
            description: NonEmptyString::from_str("Despesa").unwrap(),
            installment_id: None,
            installment_number: None,
            tags: vec![],
        },
    )
    .await
    .unwrap();

    // Income (Refund)
    Transaction::insert(
        &mut *db_tx,
        NewTransaction {
            category_id: Some(cat_id),
            source: TransactionSource::Account { account_id: acc_id },
            transaction_type: TransactionType::Income,
            amount: PositiveAmount::from_str("50.0").unwrap(),
            date: chrono::NaiveDate::from_ymd_opt(2023, 1, 16).unwrap(),
            description: NonEmptyString::from_str("Reembolso").unwrap(),
            installment_id: None,
            installment_number: None,
            tags: vec![],
        },
    )
    .await
    .unwrap();

    db_tx.commit().await.unwrap();

    let spent = budget
        .current_spend(&pool, chrono::NaiveDate::from_ymd_opt(2023, 1, 20).unwrap())
        .await
        .unwrap();

    // Spent should be 150 - 50 = 100
    assert_eq!(spent, Decimal::from_str("100.00").unwrap());
}
