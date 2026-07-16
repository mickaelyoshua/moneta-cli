use chrono::NaiveDate;
use moneta_cli::models::account::{Account, NewAccount};
use moneta_cli::models::credit_card::{CreditCard, NewCreditCard};
use moneta_cli::models::invoice::Invoice;
use moneta_cli::models::types::{
    AccountType, DayOfMonth, InvoiceStatus, Month, NonEmptyString, NonNegativeAmount, Year,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test]
async fn test_invoice_lifecycle(pool: PgPool) {
    // 1. Arrange: Conta e Cartão
    let acc = Account::insert(
        &pool,
        NewAccount {
            name: NonEmptyString::from_str("Conta Corrente").unwrap(),
            account_type: AccountType::Checking,
            has_debit_card: true,
            active: true,
        },
    )
    .await
    .unwrap();

    let card = CreditCard::insert(
        &pool,
        NewCreditCard {
            account_id: acc.id,
            name: NonEmptyString::from_str("Cartão").unwrap(),
            credit_limit: NonNegativeAmount::from_str("5000.00").unwrap(),
            billing_day: DayOfMonth::from_str("25").unwrap(), // Fecha dia 25
            due_day: DayOfMonth::from_str("5").unwrap(),      // Vence dia 5
            active: true,
        },
    )
    .await
    .unwrap();

    // 2. Act: Criação da Fatura baseado na data da transação
    // Compra no dia 20/07 (antes do fechamento 25/07) -> Fatura deve ser de Julho e vencer em 05/08
    let tx_date = NaiveDate::from_ymd_opt(2026, 7, 20).unwrap();
    let mut tx_conn = pool.begin().await.unwrap();
    let invoice1 = Invoice::find_or_create_for_date(&mut tx_conn, card.id, tx_date)
        .await
        .unwrap();
    tx_conn.commit().await.unwrap();

    assert_eq!(invoice1.month, Month::try_from(7).unwrap());
    assert_eq!(invoice1.year, Year::try_from(2026).unwrap());
    assert_eq!(
        invoice1.due_date,
        NaiveDate::from_ymd_opt(2026, 8, 5).unwrap()
    );
    assert_eq!(invoice1.status, InvoiceStatus::Open);

    // 3. Compra no dia 26/07 (depois do fechamento 25/07) -> Fatura deve ser de Agosto e vencer em 05/09
    let tx_date_next = NaiveDate::from_ymd_opt(2026, 7, 26).unwrap();
    let mut tx_conn2 = pool.begin().await.unwrap();
    let invoice2 = Invoice::find_or_create_for_date(&mut tx_conn2, card.id, tx_date_next)
        .await
        .unwrap();
    tx_conn2.commit().await.unwrap();

    assert_eq!(invoice2.month, Month::try_from(8).unwrap());
    assert_eq!(invoice2.year, Year::try_from(2026).unwrap());
    assert_eq!(
        invoice2.due_date,
        NaiveDate::from_ymd_opt(2026, 9, 5).unwrap()
    );
    assert_eq!(invoice2.status, InvoiceStatus::Open);

    // 3.5. Compra EXATAMENTE no dia do fechamento (25/07) -> Fatura deve ser de Agosto
    let tx_date_boundary = NaiveDate::from_ymd_opt(2026, 7, 25).unwrap();
    let mut tx_conn3 = pool.begin().await.unwrap();
    let invoice_boundary =
        Invoice::find_or_create_for_date(&mut tx_conn3, card.id, tx_date_boundary)
            .await
            .unwrap();
    tx_conn3.commit().await.unwrap();
    assert_eq!(
        invoice_boundary.month, Month::try_from(8).unwrap(),
        "Compras no dia do fechamento entram na próxima fatura"
    );

    // 4. Fechar Fatura 1
    let closed_invoice = Invoice::close(&pool, card.id, Month::try_from(7).unwrap(), Year::try_from(2026).unwrap()).await.unwrap();
    assert_eq!(closed_invoice.status, InvoiceStatus::Closed);

    // 5. Pagar Fatura 1 (debita da conta)
    // Atualizar total da fatura artificialmente para testar o pagamento
    sqlx::query!(
        "UPDATE invoices SET closing_amount = $1 WHERE id = $2",
        Decimal::from_str("500.00").unwrap(),
        closed_invoice.id
    )
    .execute(&pool)
    .await
    .unwrap();

    let paid_invoice = Invoice::pay(&pool, card.id, Month::try_from(7).unwrap(), Year::try_from(2026).unwrap(), acc.id).await.unwrap();
    assert_eq!(paid_invoice.status, InvoiceStatus::Paid);

    // O saldo da conta deve ter diminuído em 500
    let balance = Account::balance(&pool, acc.id).await.unwrap();
    assert_eq!(balance.balance, Decimal::from_str("-500.00").unwrap());
}

#[sqlx::test]
async fn test_invoice_generation_edge_cases(pool: PgPool) {
    let acc = Account::insert(
        &pool,
        NewAccount {
            name: NonEmptyString::from_str("Conta Edge").unwrap(),
            account_type: AccountType::Checking,
            has_debit_card: true,
            active: true,
        },
    )
    .await
    .unwrap();

    let card = CreditCard::insert(
        &pool,
        NewCreditCard {
            account_id: acc.id,
            name: NonEmptyString::from_str("Cartão Edge").unwrap(),
            credit_limit: NonNegativeAmount::from_str("5000.00").unwrap(),
            billing_day: DayOfMonth::from_str("25").unwrap(),
            due_day: DayOfMonth::from_str("28").unwrap(),
            active: true,
        },
    )
    .await
    .unwrap();

    // Transaction on Sept 20 -> Before Sept 25. Invoice is Sept. Due date is Sept 28.
    let tx_date = NaiveDate::from_ymd_opt(2026, 9, 20).unwrap();
    let mut tx_conn = pool.begin().await.unwrap();
    let invoice = Invoice::find_or_create_for_date(&mut tx_conn, card.id, tx_date)
        .await
        .unwrap();
    tx_conn.commit().await.unwrap();

    assert_eq!(invoice.month, Month::try_from(9).unwrap());
    assert_eq!(invoice.due_date, NaiveDate::from_ymd_opt(2026, 9, 28).unwrap());

    // Transaction on Nov 26 -> After Nov 25. Invoice is Dec. Due date is Dec 31.
    let tx_date = NaiveDate::from_ymd_opt(2026, 11, 26).unwrap();
    let mut tx_conn = pool.begin().await.unwrap();
    let invoice2 = Invoice::find_or_create_for_date(&mut tx_conn, card.id, tx_date)
        .await
        .unwrap();
    tx_conn.commit().await.unwrap();

    assert_eq!(invoice2.month, Month::try_from(12).unwrap());
    assert_eq!(invoice2.due_date, NaiveDate::from_ymd_opt(2026, 12, 28).unwrap());

    // Transaction on Dec 26 -> After Dec 25. Invoice is Jan (next year). Due date is Jan 28.
    let tx_date = NaiveDate::from_ymd_opt(2026, 12, 26).unwrap();
    let mut tx_conn = pool.begin().await.unwrap();
    let invoice3 = Invoice::find_or_create_for_date(&mut tx_conn, card.id, tx_date)
        .await
        .unwrap();
    tx_conn.commit().await.unwrap();

    assert_eq!(invoice3.month, Month::try_from(1).unwrap());
    assert_eq!(invoice3.year, Year::try_from(2027).unwrap());
    assert_eq!(invoice3.due_date, NaiveDate::from_ymd_opt(2027, 1, 28).unwrap());
}
