use chrono::NaiveDate;
use moneta_cli::models::account::{Account, NewAccount};
use moneta_cli::models::credit_card::{CreditCard, NewCreditCard};
use moneta_cli::models::invoice::Invoice;
use moneta_cli::models::types::{AccountType, DayOfMonth, InvoiceStatus, NonEmptyString, NonNegativeAmount};
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
    let invoice1 = Invoice::find_or_create_for_date(&pool, card.id, tx_date).await.unwrap();

    assert_eq!(invoice1.month, 7);
    assert_eq!(invoice1.year, 2026);
    assert_eq!(invoice1.due_date, NaiveDate::from_ymd_opt(2026, 8, 5).unwrap());
    assert_eq!(invoice1.status, InvoiceStatus::Open);

    // 3. Compra no dia 26/07 (depois do fechamento 25/07) -> Fatura deve ser de Agosto e vencer em 05/09
    let tx_date_next = NaiveDate::from_ymd_opt(2026, 7, 26).unwrap();
    let invoice2 = Invoice::find_or_create_for_date(&pool, card.id, tx_date_next).await.unwrap();

    assert_eq!(invoice2.month, 8);
    assert_eq!(invoice2.year, 2026);
    assert_eq!(invoice2.due_date, NaiveDate::from_ymd_opt(2026, 9, 5).unwrap());
    assert_eq!(invoice2.status, InvoiceStatus::Open);

    // 3.5. Compra EXATAMENTE no dia do fechamento (25/07) -> Fatura deve ser de Agosto
    let tx_date_boundary = NaiveDate::from_ymd_opt(2026, 7, 25).unwrap();
    let invoice_boundary = Invoice::find_or_create_for_date(&pool, card.id, tx_date_boundary).await.unwrap();
    assert_eq!(invoice_boundary.month, 8, "Compras no dia do fechamento entram na próxima fatura");

    // 4. Fechar Fatura 1
    let closed_invoice = Invoice::close(&pool, card.id, 7, 2026).await.unwrap();
    assert_eq!(closed_invoice.status, InvoiceStatus::Closed);

    // 5. Pagar Fatura 1 (debita da conta)
    // Atualizar total da fatura artificialmente para testar o pagamento
    sqlx::query!("UPDATE invoices SET total_amount = $1 WHERE id = $2", Decimal::from_str("500.00").unwrap(), closed_invoice.id)
        .execute(&pool).await.unwrap();

    let paid_invoice = Invoice::pay(&pool, card.id, 7, 2026, acc.id).await.unwrap();
    assert_eq!(paid_invoice.status, InvoiceStatus::Paid);

    // O saldo da conta deve ter diminuído em 500
    let balance = Account::balance(&pool, acc.id).await.unwrap();
    assert_eq!(balance.balance, Decimal::from_str("-500.00").unwrap());
}
