use moneta_cli::models::transaction::{NewTransaction, Transaction};
use moneta_cli::models::types::{
    NonEmptyString, PositiveAmount, TransactionSource, TransactionType,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::str::FromStr;

#[sqlx::test]
async fn test_transaction_insert_and_find_all(pool: PgPool) {
    // Setup - Injecting dependencies via dynamic SQL to avoid requiring compile-time sqlx prepare
    let ctg_id: i32 = sqlx::query(
        "INSERT INTO categories (name, category_type) VALUES ('Food', 'expense') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("setup category")
    .get("id");

    let account_id: i32 = sqlx::query(
        "INSERT INTO accounts (name, account_type) VALUES ('Nubank', 'checking') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("setup account")
    .get("id");

    let new_tx = NewTransaction {
        category_id: Some(ctg_id),
        source: TransactionSource::Account { account_id },
        transaction_type: TransactionType::Expense,
        amount: PositiveAmount::from_str("15.50").unwrap(),
        date: chrono::NaiveDate::from_ymd_opt(2026, 7, 7).unwrap(),
        description: NonEmptyString::from_str("Lunch").unwrap(),
        installment_id: None,
        installment_number: None,
        tags: vec![],
    };

    let mut tx = pool.begin().await.expect("start tx");

    // Action
    let transaction = Transaction::insert(&mut tx, new_tx)
        .await
        .expect("Failed to insert transaction");
        
    tx.commit().await.expect("commit tx");

    // Assertion - Verify mapping to database shape
    assert_eq!(transaction.category_id, Some(ctg_id));
    assert_eq!(transaction.source, TransactionSource::Account { account_id });
    assert_eq!(transaction.transaction_type, TransactionType::Expense);
    assert_eq!(transaction.amount.as_decimal(), Decimal::new(1550, 2));
    assert_eq!(transaction.description.as_str(), "Lunch");
    assert_eq!(
        transaction.date,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 7).unwrap()
    );

    // Action 2
    let all = Transaction::find_all(&pool, None)
        .await
        .expect("Failed to find_all");

    // Assertion 2
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, transaction.id);
}

#[sqlx::test]
async fn test_transaction_update_and_delete(pool: PgPool) {
    // Setup
    let ctg_id: i32 = sqlx::query(
        "INSERT INTO categories (name, category_type) VALUES ('Food', 'expense') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("setup category")
    .get("id");

    let account_id: i32 = sqlx::query(
        "INSERT INTO accounts (name, account_type) VALUES ('Nubank', 'checking') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("setup account")
    .get("id");
    
    let credit_card_id: i32 = sqlx::query(
        "INSERT INTO credit_cards (account_id, name, credit_limit, billing_day, due_day) VALUES ($1, 'Card', 1000, 25, 5) RETURNING id",
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .unwrap()
    .get("id");

    let new_tx = NewTransaction {
        category_id: Some(ctg_id),
        source: TransactionSource::Account { account_id },
        transaction_type: TransactionType::Expense,
        amount: PositiveAmount::from_str("10.00").unwrap(),
        date: chrono::NaiveDate::from_ymd_opt(2026, 7, 7).unwrap(),
        description: NonEmptyString::from_str("Lanche").unwrap(),
        installment_id: None,
        installment_number: None,
        tags: vec![],
    };

    let mut db_tx = pool.begin().await.unwrap();
    let tx = Transaction::insert(&mut db_tx, new_tx).await.unwrap();
    db_tx.commit().await.unwrap();

    // UPDATE: Mover para o Cartão de Crédito com novo valor
    let mut db_tx = pool.begin().await.unwrap();
    let payload = moneta_cli::models::transaction::UpdateTransactionPayload {
        account_id: None,
        credit_card_id: Some(credit_card_id),
        category_id: None,
        transaction_type: None,
        amount: Some(PositiveAmount::from_str("20.00").unwrap()),
        date: None,
        description: Some(NonEmptyString::from_str("Jantar").unwrap()),
    };
    
    let updated = Transaction::update(&mut db_tx, tx.id, payload).await.unwrap();
    db_tx.commit().await.unwrap();

    assert_eq!(updated.amount.as_decimal(), rust_decimal::Decimal::from_str("20.00").unwrap());
    assert_eq!(updated.description.as_str(), "Jantar");
    assert!(updated.invoice_id.is_some(), "Deveria ter gerado invoice_id");
    
    // DELETE
    Transaction::delete(&pool, tx.id).await.unwrap();
    
    let find = Transaction::find_by_id(&pool, tx.id).await;
    assert!(find.is_err(), "Deveria ter sido deletada");
}

#[sqlx::test]
async fn test_transaction_update_delete_constraints(pool: PgPool) {
    let account_id: i32 = sqlx::query(
        "INSERT INTO accounts (name, account_type) VALUES ('BB', 'checking') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap()
    .get("id");

    let ctg_id: i32 = sqlx::query(
        "INSERT INTO categories (name, category_type) VALUES ('Lazer', 'expense') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap()
    .get("id");
    
    let card_id: i32 = sqlx::query(
        "INSERT INTO credit_cards (account_id, name, credit_limit, billing_day, due_day) VALUES ($1, 'Card2', 1000, 25, 5) RETURNING id",
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .unwrap()
    .get("id");

    // Teste 1: Installment Constraint
    let inst = moneta_cli::models::installment::Installment::insert(
        &pool,
        moneta_cli::models::installment::NewInstallment {
            credit_card_id: card_id,
            category_id: Some(ctg_id),
            description: NonEmptyString::from_str("TV").unwrap(),
            total_amount: PositiveAmount::from_str("100.00").unwrap(),
            installments_count: 2,
            date: chrono::NaiveDate::from_ymd_opt(2026, 7, 7).unwrap(),
        },
    ).await.unwrap();

    let mut txs = Transaction::find_all(&pool, None).await.unwrap();
    let inst_tx = txs.pop().unwrap();

    // Tentar atualizar
    let mut db_tx = pool.begin().await.unwrap();
    let err_update = Transaction::update(
        &mut db_tx,
        inst_tx.id,
        moneta_cli::models::transaction::UpdateTransactionPayload {
            amount: Some(PositiveAmount::from_str("50.00").unwrap()),
            account_id: None, credit_card_id: None, category_id: None, transaction_type: None, date: None, description: None
        }
    ).await;
    assert!(err_update.is_err(), "Deveria barrar update em transaction originada de installment");
    db_tx.rollback().await.unwrap();

    // Tentar deletar
    let err_delete = Transaction::delete(&pool, inst_tx.id).await;
    assert!(err_delete.is_err(), "Deveria barrar delete em transaction originada de installment");

    // Teste 2: Invoice Closed Constraint
    // Deletar o installment antes pra limpar as faturas
    moneta_cli::models::installment::Installment::delete(&pool, inst.id).await.unwrap();

    let new_tx = NewTransaction {
        category_id: Some(ctg_id),
        source: TransactionSource::CreditCard { credit_card_id: card_id },
        transaction_type: TransactionType::Expense,
        amount: PositiveAmount::from_str("20.00").unwrap(),
        date: chrono::NaiveDate::from_ymd_opt(2026, 7, 7).unwrap(),
        description: NonEmptyString::from_str("Lanche").unwrap(),
        installment_id: None,
        installment_number: None,
        tags: vec![],
    };

    let mut db_tx = pool.begin().await.unwrap();
    let tx_normal = Transaction::insert(&mut db_tx, new_tx).await.unwrap();
    db_tx.commit().await.unwrap();

    // Fechar fatura
    moneta_cli::models::invoice::Invoice::close(&pool, card_id, 7, 2026).await.unwrap();

    // Tentar deletar
    let err_delete_closed = Transaction::delete(&pool, tx_normal.id).await;
    assert!(err_delete_closed.is_err(), "Deveria barrar delete em fatura fechada");

    // Tentar update
    let mut db_tx = pool.begin().await.unwrap();
    let err_update_closed = Transaction::update(
        &mut db_tx,
        tx_normal.id,
        moneta_cli::models::transaction::UpdateTransactionPayload {
            amount: Some(PositiveAmount::from_str("30.00").unwrap()),
            account_id: None, credit_card_id: None, category_id: None, transaction_type: None, date: None, description: None
        }
    ).await;
    assert!(err_update_closed.is_err(), "Deveria barrar update em fatura fechada");
}
