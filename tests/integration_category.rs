use moneta_cli::models::category::{Category, NewCategory};
use moneta_cli::models::types::{CategoryType, NonEmptyString};
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_category_crud(pool: PgPool) {
    let new_ctg1 = NewCategory {
        name: NonEmptyString::from_str("Test Income").unwrap(),
        category_type: CategoryType::Income,
        active: true,
    };

    let new_ctg2 = NewCategory {
        name: NonEmptyString::from_str("Test Expense").unwrap(),
        category_type: CategoryType::Expense,
        active: false,
    };

    let ctg1 = Category::insert(&pool, new_ctg1)
        .await
        .expect("Failed to insert ctg1");
    let ctg2 = Category::insert(&pool, new_ctg2)
        .await
        .expect("Failed to insert ctg2");

    assert_eq!(ctg1.name.as_str(), "Test Income");
    assert_eq!(ctg1.category_type, CategoryType::Income);
    assert!(ctg1.active);

    assert_eq!(ctg2.name.as_str(), "Test Expense");
    assert_eq!(ctg2.category_type, CategoryType::Expense);
    assert!(!ctg2.active);

    let all = Category::find_all(&pool, None)
        .await
        .expect("Failed to find_all");

    assert_eq!(all.len(), 2);
    let names: Vec<&str> = all.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"Test Income"));
    assert!(names.contains(&"Test Expense"));

    // Update
    let mut to_update = Category::find_by_id(&pool, ctg1.id)
        .await
        .expect("Should find by id");
    to_update.name = NonEmptyString::from_str("Updated Income").unwrap();
    to_update.active = false;
    let updated = to_update.update(&pool).await.expect("Failed to update");

    assert_eq!(updated.name.as_str(), "Updated Income");
    assert!(!updated.active);

    // Delete
    let deleted = Category::delete(&pool, ctg2.id)
        .await
        .expect("Failed to delete");
    assert!(deleted);

    let all_after_delete = Category::find_all(&pool, None)
        .await
        .expect("Failed to find_all");
    assert_eq!(all_after_delete.len(), 1);
}
