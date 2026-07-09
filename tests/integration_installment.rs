use chrono::NaiveDate;
use moneta_cli::models::account::{Account, NewAccount};
use moneta_cli::models::category::{Category, NewCategory};
use moneta_cli::models::credit_card::{CreditCard, NewCreditCard};
use moneta_cli::models::installment::{Installment, NewInstallment};
use moneta_cli::models::invoice::Invoice;
use moneta_cli::models::transaction::Transaction;
use moneta_cli::models::types::{
    AccountType, CategoryType, DayOfMonth, NonEmptyString, NonNegativeAmount, PositiveAmount,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;

async fn setup_test_data(pool: &PgPool) -> (i32, i32) {
    let acc = Account::insert(
        pool,
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
        pool,
        NewCreditCard {
            account_id: acc.id,
            name: NonEmptyString::from_str("Cartão").unwrap(),
            credit_limit: NonNegativeAmount::from_str("5000.00").unwrap(),
            billing_day: DayOfMonth::from_str("25").unwrap(),
            due_day: DayOfMonth::from_str("5").unwrap(),
            active: true,
        },
    )
    .await
    .unwrap();

    let ctg = Category::insert(
        pool,
        NewCategory {
            name: NonEmptyString::from_str("Eletrônicos").unwrap(),
            category_type: CategoryType::Expense,
            active: true,
        },
    )
    .await
    .unwrap();

    (card.id, ctg.id)
}

#[sqlx::test]
async fn test_installment_insert_math(pool: PgPool) {
    let (card_id, ctg_id) = setup_test_data(&pool).await;

    // Criar um parcelamento de 10.00 em 3 vezes (deve gerar 3.33, 3.33, 3.34)
    let inst = Installment::insert(
        &pool,
        NewInstallment {
            credit_card_id: card_id,
            category_id: Some(ctg_id),
            description: NonEmptyString::from_str("Mouse").unwrap(),
            total_amount: PositiveAmount::from_str("10.00").unwrap(),
            installments_count: 3,
            date: NaiveDate::from_ymd_opt(2026, 7, 20).unwrap(),
        },
    )
    .await
    .unwrap();

    assert_eq!(inst.installments_count, 3);
    assert_eq!(inst.total_amount.as_decimal(), Decimal::from_str("10.00").unwrap());

    // Verificar as transações geradas
    let mut txs = Transaction::find_all(&pool, None).await.unwrap();
    assert_eq!(txs.len(), 3);

    // Ordenar por data
    txs.sort_by(|a, b| a.date.cmp(&b.date));

    // Parcela 1
    assert_eq!(txs[0].amount.as_decimal(), Decimal::from_str("3.33").unwrap());
    assert_eq!(txs[0].installment_number, Some(1));
    assert_eq!(txs[0].date, NaiveDate::from_ymd_opt(2026, 7, 20).unwrap());

    // Parcela 2
    assert_eq!(txs[1].amount.as_decimal(), Decimal::from_str("3.33").unwrap());
    assert_eq!(txs[1].installment_number, Some(2));
    assert_eq!(txs[1].date, NaiveDate::from_ymd_opt(2026, 8, 20).unwrap());

    // Parcela 3 (resto vai para a última)
    assert_eq!(txs[2].amount.as_decimal(), Decimal::from_str("3.34").unwrap());
    assert_eq!(txs[2].installment_number, Some(3));
    assert_eq!(txs[2].date, NaiveDate::from_ymd_opt(2026, 9, 20).unwrap());
}

#[sqlx::test]
async fn test_installment_adjust_and_delete(pool: PgPool) {
    let (card_id, ctg_id) = setup_test_data(&pool).await;

    // Criar um parcelamento
    let inst = Installment::insert(
        &pool,
        NewInstallment {
            credit_card_id: card_id,
            category_id: Some(ctg_id),
            description: NonEmptyString::from_str("Teste").unwrap(),
            total_amount: PositiveAmount::from_str("100.00").unwrap(),
            installments_count: 2, // 50.00 e 50.00
            date: NaiveDate::from_ymd_opt(2026, 7, 20).unwrap(),
        },
    )
    .await
    .unwrap();

    // 1. Ajustar a primeira parcela para 49.99
    let adjusted_tx = Installment::adjust(
        &pool,
        inst.id,
        1,
        PositiveAmount::from_str("49.99").unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(adjusted_tx.amount.as_decimal(), Decimal::from_str("49.99").unwrap());

    // Verificar current_total da fatura
    let current_total = Invoice::current_total(&pool, adjusted_tx.invoice_id.unwrap()).await.unwrap();
    assert_eq!(current_total, Decimal::from_str("49.99").unwrap());

    // 2. Fechar a fatura da primeira parcela
    Invoice::close(&pool, card_id, 7, 2026).await.unwrap();

    // 3. Tentar deletar o parcelamento (deve falhar pois a fatura 1 está fechada)
    let delete_result = Installment::delete(&pool, inst.id).await;
    assert!(delete_result.is_err(), "Não deve permitir deletar parcelamento com fatura fechada");

    // 4. Tentar ajustar a primeira parcela de novo (deve falhar pois a fatura está fechada)
    let adjust_result2 = Installment::adjust(
        &pool,
        inst.id,
        1,
        PositiveAmount::from_str("40.00").unwrap(),
    )
    .await;
    assert!(adjust_result2.is_err(), "Não deve permitir ajustar parcela em fatura fechada");
}

#[sqlx::test]
async fn test_installment_insert_math_edge_case(pool: PgPool) {
    let (card_id, ctg_id) = setup_test_data(&pool).await;

    // Criar um parcelamento de 0.05 em 10 vezes (impossível, deve retornar Erro ou corrigir perfeitamente)
    // O sistema atual vai criar parcelas de 0.01 e explodir a soma final para 0.10, criando 0.05 de dívida fantasma.
    let inst_result = Installment::insert(
        &pool,
        NewInstallment {
            credit_card_id: card_id,
            category_id: Some(ctg_id),
            description: NonEmptyString::from_str("Bala").unwrap(),
            total_amount: PositiveAmount::from_str("0.05").unwrap(),
            installments_count: 10,
            date: NaiveDate::from_ymd_opt(2026, 7, 20).unwrap(),
        },
    )
    .await;

    // Deve ser um erro! Regra de negócio: Total Amount deve ser no mínimo 1 centavo por parcela (0.10)
    assert!(inst_result.is_err(), "Deve falhar ao tentar parcelar valor menor que a quantidade de parcelas em centavos");
}
