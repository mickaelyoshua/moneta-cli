use moneta_cli::context::AppContext;
use moneta_cli::db::Db;
use moneta_cli::commands::import::ImportCommand;
use moneta_cli::handlers::import::process_csv;
use moneta_cli::models::account::{Account, NewAccount};
use moneta_cli::models::credit_card::{CreditCard, NewCreditCard};
use moneta_cli::models::types::{AccountType, NonEmptyString, NonNegativeAmount, DayOfMonth};
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_integration_import_csv(pool: PgPool) {
    let ctx = AppContext {
        db: Db { pool: pool.clone() },
        json_output: false,
    };

    // 1. Setup Data
    let acc = Account::insert(
        &pool,
        NewAccount {
            name: NonEmptyString::from_str("Conta Principal").unwrap(),
            account_type: AccountType::Checking,
            has_debit_card: false,
            active: true,
        },
    )
    .await
    .unwrap();

    let _cc = CreditCard::insert(
        &pool,
        NewCreditCard {
            account_id: acc.id,
            name: NonEmptyString::from_str("Nubank").unwrap(),
            credit_limit: NonNegativeAmount::from_str("1000").unwrap(),
            billing_day: DayOfMonth::from_str("1").unwrap(),
            due_day: DayOfMonth::from_str("10").unwrap(),
            active: true,
        },
    )
    .await
    .unwrap();

    // 2. Prepare CSV
    let csv_content = r#"date,amount,description,category,tags,installments,source_type,source_name
2023-01-01,50.0,Ifood,Alimentação,tag1, ,account,Conta Principal
2023-01-02,100.0,TV Nova,Eletrônicos, ,3/5,credit_card,Nubank
"#;

    let temp_path = std::env::temp_dir().join("test_import.csv");
    std::fs::write(&temp_path, csv_content).unwrap();

    // 3. Process
    let cmd = ImportCommand {
        file: temp_path.clone(),
        dry_run: false,
    };

    let result = process_csv(&ctx, cmd).await.expect("Failed to process CSV");

    assert_eq!(result.total_rows, 2);
    assert_eq!(result.imported, 2);
    assert_eq!(result.skipped, 0);

    // 4. Verify DB
    let txs = moneta_cli::models::transaction::Transaction::find_all(&pool, None).await.unwrap();
    let installs = moneta_cli::models::installment::Installment::find_all(&pool, None).await.unwrap();
    
    // Cleanup
    let _ = std::fs::remove_file(temp_path);
    
    // 1 Ifood transaction + 5 TV Nova transactions = 6
    assert_eq!(txs.len(), 6);
    assert_eq!(installs.len(), 1);
    assert_eq!(installs[0].description.as_str(), "TV Nova");
}
