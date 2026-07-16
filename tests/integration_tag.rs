use moneta_cli::models::transaction::{NewTransaction, Transaction};
use moneta_cli::models::types::{
    NonEmptyString, PositiveAmount, TransactionSource, TransactionType,
};
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_tag_resolution_on_insert(pool: PgPool) {
    let cat_id = sqlx::query!(
        "INSERT INTO categories (name, category_type) VALUES ('Teste', 'expense') RETURNING id"
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

    let new_tx = NewTransaction {
        category_id: Some(cat_id),
        source: TransactionSource::Account { account_id: acc_id },
        transaction_type: TransactionType::Expense,
        amount: PositiveAmount::from_str("100.0").unwrap(),
        date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        description: NonEmptyString::from_str("Test with tags").unwrap(),
        installment_id: None,
        installment_number: None,
        tags: vec![
            moneta_cli::models::types::NonEmptyString::from_str("lazer").unwrap(),
            moneta_cli::models::types::NonEmptyString::from_str("viagem").unwrap(),
        ],
    };

    let mut db_tx = pool.begin().await.unwrap();
    let tx = Transaction::insert(&mut db_tx, new_tx)
        .await
        .expect("Insert failed");
    db_tx.commit().await.unwrap();

    let all = Transaction::find_all(&pool, None)
        .await
        .expect("List failed");
    let saved_tx = all.into_iter().find(|t| t.id == tx.id).unwrap();

    assert_eq!(saved_tx.tags.len(), 2);
    assert!(saved_tx.tags.contains(&moneta_cli::models::types::NonEmptyString::from_str("lazer").unwrap()));
    assert!(saved_tx.tags.contains(&moneta_cli::models::types::NonEmptyString::from_str("viagem").unwrap()));
}
