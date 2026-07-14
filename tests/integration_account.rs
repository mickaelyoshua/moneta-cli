use moneta_cli::models::account::Account;
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_account_crud(pool: PgPool) {
    let name = "Test Account";
    let account_type = moneta_cli::models::types::AccountType::Checking;
    let has_debit_card = true;
    let active = true;

    // Simulate insert
    let account = sqlx::query_as::<_, Account>(
        r#"
        INSERT INTO accounts (name, account_type, has_debit_card, active)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(account_type)
    .bind(has_debit_card)
    .bind(active)
    .fetch_one(&pool)
    .await
    .expect("Failed to insert account");

    assert_eq!(account.name.as_str(), "Test Account");
    assert_eq!(account.account_type, account_type);
    assert!(account.has_debit_card);
    assert!(account.active);

    // Simulate fetch
    let accounts = sqlx::query_as::<_, Account>(
        r#"
        SELECT * FROM accounts
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch accounts");

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, account.id);

    // find_by_id
    let mut fetched_account = Account::find_by_id(&pool, account.id)
        .await
        .expect("Failed to find_by_id");
    assert_eq!(fetched_account.name.as_str(), "Test Account");

    // update
    fetched_account.name =
        moneta_cli::models::types::NonEmptyString::from_str("Updated Account").unwrap();
    let updated_account = fetched_account
        .update(&pool)
        .await
        .expect("Failed to update account");
    assert_eq!(updated_account.name.as_str(), "Updated Account");

    // delete
    let deleted = Account::delete(&pool, account.id)
        .await
        .expect("Failed to delete account");
    assert!(deleted);

    let accounts_after_delete = Account::find_all(&pool, None)
        .await
        .expect("Failed to fetch accounts");
    assert_eq!(accounts_after_delete.len(), 0);
}
