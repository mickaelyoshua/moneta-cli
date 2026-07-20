use moneta_cli::models::recurrence::{NewRecurrence, Recurrence};
use moneta_cli::models::transaction::Transaction;
use moneta_cli::models::types::{
    NonEmptyString, PositiveAmount, RecurrenceFrequency, TransactionSource, TransactionType,
};
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_recurrence_sync(pool: PgPool) {
    let cat_id = sqlx::query!(
        "INSERT INTO categories (name, category_type) VALUES ('Internet', 'expense') RETURNING id"
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

    let new_rec = NewRecurrence {
        category_id: cat_id,
        source: TransactionSource::Account { account_id: acc_id },
        transaction_type: TransactionType::Expense,
        amount: PositiveAmount::from_str("100.0").unwrap(),
        description: NonEmptyString::from_str("Mensalidade").unwrap(),
        frequency: RecurrenceFrequency::Monthly,
        start_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
        end_date: None,
    };

    let rec = Recurrence::insert(&pool, new_rec).await.unwrap();

    let ref_date = chrono::NaiveDate::from_ymd_opt(2023, 3, 15).unwrap();
    let inserted = Recurrence::sync_all(&pool, ref_date).await.unwrap();

    assert_eq!(inserted, 3);

    let all = Transaction::find_all(&pool, None, None).await.unwrap();
    assert_eq!(all.len(), 3);
    assert!(all.iter().all(|t| t.recurrence_id == Some(rec.id)));

    let inserted2 = Recurrence::sync_all(&pool, ref_date).await.unwrap();
    assert_eq!(inserted2, 0);
}

#[sqlx::test]
async fn test_recurrence_end_of_month_sync(pool: PgPool) {
    let cat_id = sqlx::query!(
        "INSERT INTO categories (name, category_type) VALUES ('Internet', 'expense') RETURNING id"
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

    let new_rec = NewRecurrence {
        category_id: cat_id,
        source: TransactionSource::Account { account_id: acc_id },
        transaction_type: TransactionType::Expense,
        amount: PositiveAmount::from_str("100.0").unwrap(),
        description: NonEmptyString::from_str("Mensalidade 31").unwrap(),
        frequency: RecurrenceFrequency::Monthly,
        start_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
        end_date: None,
    };

    let _rec = Recurrence::insert(&pool, new_rec).await.unwrap();

    let ref_date = chrono::NaiveDate::from_ymd_opt(2023, 4, 1).unwrap();
    let inserted = Recurrence::sync_all(&pool, ref_date).await.unwrap();

    assert_eq!(inserted, 3);

    let mut all = Transaction::find_all(&pool, None, None).await.unwrap();
    all.sort_by_key(|t| t.date);

    assert_eq!(all.len(), 3);
    assert_eq!(all[0].date, chrono::NaiveDate::from_ymd_opt(2023, 1, 31).unwrap());
    assert_eq!(all[1].date, chrono::NaiveDate::from_ymd_opt(2023, 2, 28).unwrap());
    assert_eq!(all[2].date, chrono::NaiveDate::from_ymd_opt(2023, 3, 31).unwrap());
}
