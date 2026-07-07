use moneta_cli::models::category::{Category, NewCategory};
use moneta_cli::models::types::{CategoryType, NonEmptyString};
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_category_crud(pool: PgPool) {
    let new_cat1 = NewCategory {
        name: NonEmptyString::from_str("Test Income").unwrap(),
        category_type: CategoryType::Income,
        active: true,
    };
    
    let new_cat2 = NewCategory {
        name: NonEmptyString::from_str("Test Expense").unwrap(),
        category_type: CategoryType::Expense,
        active: false,
    };

    let cat1 = Category::insert(&pool, new_cat1).await.expect("Failed to insert cat1");
    let cat2 = Category::insert(&pool, new_cat2).await.expect("Failed to insert cat2");

    assert_eq!(cat1.name.as_str(), "Test Income");
    assert_eq!(cat1.category_type, CategoryType::Income);
    assert_eq!(cat1.active, true);
    
    assert_eq!(cat2.name.as_str(), "Test Expense");
    assert_eq!(cat2.category_type, CategoryType::Expense);
    assert_eq!(cat2.active, false);

    let all = Category::find_all(&pool, None).await.expect("Failed to find_all");
    
    assert_eq!(all.len(), 2);
    let names: Vec<&str> = all.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"Test Income"));
    assert!(names.contains(&"Test Expense"));
}
