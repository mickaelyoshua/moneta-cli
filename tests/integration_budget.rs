use moneta_cli::models::budget::Budget;
use moneta_cli::models::transaction::{NewTransaction, Transaction};
use moneta_cli::models::types::{
    BudgetPeriod, NonEmptyString, PositiveAmount, TransactionSource, TransactionType,
};
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_budget_creation_and_constraints(pool: PgPool) {
    // Deve falhar pois requer category ou tag
    let res = Budget::insert(
        &pool,
        None,
        None,
        PositiveAmount::from_str("100.0").unwrap(),
        BudgetPeriod::Monthly,
    )
    .await;
    assert!(res.is_err());
}

#[sqlx::test]
async fn test_budget_current_spend(pool: PgPool) {
    let cat_id = sqlx::query!(
        "INSERT INTO categories (name, category_type) VALUES ('Lazer', 'expense') RETURNING id"
    )
    .fetch_one(&pool)
    .await
    .unwrap()
    .id;
    let acc_id = sqlx::query!(
        "INSERT INTO accounts (name, account_type) VALUES ('Conta', 'checking') RETURNING id"
    )
    .fetch_one(&pool)
    .await
    .unwrap()
    .id;

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

    for i in 1..=2 {
        Transaction::insert(
            &mut db_tx,
            NewTransaction {
                category_id: Some(cat_id),
                source: TransactionSource::Account { account_id: acc_id },
                transaction_type: TransactionType::Expense,
                amount: PositiveAmount::from_str("150.0").unwrap(),
                date: chrono::NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
                description: NonEmptyString::from_str(&format!("Tx {}", i)).unwrap(),
                installment_id: None,
                installment_number: None,
                tags: vec![],
            },
        )
        .await
        .unwrap();
    }

    Transaction::insert(
        &mut db_tx,
        NewTransaction {
            category_id: Some(cat_id),
            source: TransactionSource::Account { account_id: acc_id },
            transaction_type: TransactionType::Expense,
            amount: PositiveAmount::from_str("100.0").unwrap(),
            date: chrono::NaiveDate::from_ymd_opt(2023, 2, 1).unwrap(),
            description: NonEmptyString::from_str("Tx 3").unwrap(),
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
    assert_eq!(spent, rust_decimal::Decimal::from_str("300.00").unwrap());
}
