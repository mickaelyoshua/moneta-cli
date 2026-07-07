use moneta_cli::models::account::Account;
use sqlx::PgPool;

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
}
