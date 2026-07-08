use moneta_cli::models::category::{Category, NewCategory};
use moneta_cli::models::types::{CategoryType, NonEmptyString};
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_auto_updated_at(pool: PgPool) {
    let new_ctg = NewCategory {
        name: NonEmptyString::from_str("Test Moddatetime").unwrap(),
        category_type: CategoryType::Income,
        active: true,
    };

    let inserted = Category::insert(&pool, new_ctg).await.unwrap();
    let old_updated_at = inserted.updated_at;

    // Sleep a bit
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Trigger an update
    sqlx::query!("UPDATE categories SET name = 'Updated' WHERE id = $1", inserted.id)
        .execute(&pool)
        .await
        .unwrap();

    let updated = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
        .bind(inserted.id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let new_updated_at = updated.updated_at;

    println!("Old: {}", old_updated_at);
    println!("New: {}", new_updated_at);
    
    assert!(new_updated_at > old_updated_at, "updated_at should have increased");
}
