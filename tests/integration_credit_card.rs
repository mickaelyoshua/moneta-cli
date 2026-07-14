use moneta_cli::models::account::Account;
use moneta_cli::models::credit_card::CreditCard;
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_credit_card_crud(pool: PgPool) {
    // Need an account first due to FK
    let account = sqlx::query_as::<_, Account>(
        r#"
        INSERT INTO accounts (name, account_type, has_debit_card, active)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind("Parent Account")
    .bind(moneta_cli::models::types::AccountType::Checking)
    .bind(true)
    .bind(true)
    .fetch_one(&pool)
    .await
    .expect("Failed to insert parent account");

    let card_name = "My Credit Card";
    let credit_limit = moneta_cli::models::types::NonNegativeAmount::from_str("5000.0").unwrap();
    let billing_day = 10i16;
    let due_day = 20i16;

    let card = sqlx::query_as::<_, CreditCard>(
        r#"
        INSERT INTO credit_cards (account_id, name, credit_limit, billing_day, due_day, active)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(account.id)
    .bind(card_name)
    .bind(credit_limit)
    .bind(billing_day)
    .bind(due_day)
    .bind(true)
    .fetch_one(&pool)
    .await
    .expect("Failed to insert credit card");

    assert_eq!(card.account_id, account.id);
    assert_eq!(card.name.as_str(), card_name);
    assert_eq!(card.credit_limit, credit_limit);
    assert_eq!(card.billing_day, billing_day);
    assert_eq!(card.due_day, due_day);
    assert!(card.active);

    // Simulate fetch
    let cards = sqlx::query_as::<_, CreditCard>(
        r#"
        SELECT * FROM credit_cards
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch cards");

    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].id, card.id);

    // find_by_id
    let mut fetched_card = CreditCard::find_by_id(&pool, card.id)
        .await
        .expect("Failed to find_by_id");
    assert_eq!(fetched_card.name.as_str(), "My Credit Card");

    // update
    fetched_card.name =
        moneta_cli::models::types::NonEmptyString::from_str("Updated Card").unwrap();
    let updated_card = fetched_card
        .update(&pool)
        .await
        .expect("Failed to update card");
    assert_eq!(updated_card.name.as_str(), "Updated Card");

    // delete
    let deleted = CreditCard::delete(&pool, card.id)
        .await
        .expect("Failed to delete card");
    assert!(deleted);

    let cards_after_delete = CreditCard::find_all(&pool, None)
        .await
        .expect("Failed to fetch cards");
    assert_eq!(cards_after_delete.len(), 0);
}

#[sqlx::test]
async fn test_credit_card_fk_violation(pool: PgPool) {
    let credit_limit = moneta_cli::models::types::NonNegativeAmount::from_str("5000.0").unwrap();

    // Insert with invalid account_id 9999
    let result = sqlx::query(
        r#"
        INSERT INTO credit_cards (account_id, name, credit_limit, billing_day, due_day, active)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(9999)
    .bind("Invalid Card")
    .bind(credit_limit)
    .bind(10i16)
    .bind(20i16)
    .bind(true)
    .execute(&pool)
    .await;

    assert!(
        result.is_err(),
        "Should have failed due to foreign key constraint"
    );
}
