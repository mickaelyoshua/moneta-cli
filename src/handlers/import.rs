use crate::commands::import::{ImportCommand, ImportError};
use crate::context::AppContext;
use crate::models::import::{ParsedCsvRecord, RawCsvRecord, deduplicate_records};
use crate::models::transaction::{NewTransaction, Transaction};
use crate::models::types::{TransactionSource, TransactionType};
use crate::models::{
    account::Account, category::Category, credit_card::CreditCard, installment::Installment,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub total_rows: usize,
    pub imported: usize,
    pub skipped: usize,
    pub dry_run: bool,
}

pub async fn process_csv(
    ctx: &AppContext,
    cmd: ImportCommand,
) -> Result<ImportResult, ImportError> {
    let raw_records = load_csv(&cmd.file)?;

    let parsed_records = parse_records(raw_records)?;

    if parsed_records.is_empty() {
        return Ok(ImportResult {
            total_rows: 0,
            imported: 0,
            skipped: 0,
            dry_run: cmd.dry_run,
        });
    }

    let all_txs = Transaction::find_all(&ctx.db.pool, Some(10000)).await?;
    let to_insert = deduplicate_records(&parsed_records, &all_txs);

    let skipped = parsed_records.len() - to_insert.len();
    let imported = to_insert.len();

    if !cmd.dry_run {
        let mut conn = ctx.db.pool.acquire().await?;
        for rec in to_insert {
            insert_record(ctx, &mut conn, rec).await?;
        }
    }

    Ok(ImportResult {
        total_rows: parsed_records.len(),
        imported,
        skipped,
        dry_run: cmd.dry_run,
    })
}

fn load_csv(path: &std::path::Path) -> Result<Vec<RawCsvRecord>, ImportError> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut records = Vec::new();
    for result in rdr.deserialize() {
        records.push(result?);
    }
    Ok(records)
}

fn parse_records(raw_records: Vec<RawCsvRecord>) -> Result<Vec<ParsedCsvRecord>, ImportError> {
    let mut parsed = Vec::with_capacity(raw_records.len());
    for raw in raw_records {
        parsed.push(ParsedCsvRecord::try_from(raw).map_err(ImportError::Handler)?);
    }
    Ok(parsed)
}

async fn insert_record(
    ctx: &AppContext,
    conn: &mut sqlx::PgConnection,
    rec: ParsedCsvRecord,
) -> Result<(), ImportError> {
    let source = resolve_source(ctx, rec.source_type.as_str(), rec.source_name.as_str()).await?;
    let category = Category::find_or_create(&ctx.db.pool, rec.category.as_str()).await?;

    if let Some((current, total)) = rec.installment {
        if let TransactionSource::CreditCard { credit_card_id } = source {
            let total_amount = rec.amount.as_decimal() * rust_decimal::Decimal::from(total);
            let pos_total_amount = crate::models::types::PositiveAmount::try_from(total_amount)
                .map_err(|e| ImportError::Handler(format!("Erro ao calcular parcela: {}", e)))?;
            let count = crate::models::types::InstallmentCount::try_from(total)
                .map_err(|e| ImportError::Handler(format!("Count de parcela inválido: {}", e)))?;

            let original_date = rec.date - chrono::Months::new((current - 1) as u32);

            Installment::find_or_create_for_import(
                &ctx.db.pool,
                credit_card_id,
                Some(category.id),
                rec.description,
                pos_total_amount,
                count,
                original_date,
            )
            .await?;
        }
        return Ok(());
    }

    let new_tx = NewTransaction {
        category_id: Some(category.id),
        source,
        transaction_type: TransactionType::Expense,
        amount: rec.amount,
        date: rec.date,
        description: rec.description,
        installment_id: None,
        installment_number: None,
        tags: rec.tags,
    };

    Transaction::insert(conn, new_tx).await?;
    Ok(())
}

async fn resolve_source(
    ctx: &AppContext,
    stype: &str,
    sname: &str,
) -> Result<TransactionSource, ImportError> {
    if stype == "account" {
        let acc = Account::find_by_name(&ctx.db.pool, sname).await?;
        if let Some(account) = acc {
            Ok(TransactionSource::Account {
                account_id: account.id,
            })
        } else {
            Err(ImportError::Handler(format!(
                "Conta não encontrada: {}",
                sname
            )))
        }
    } else if stype == "credit_card" {
        let cc = CreditCard::find_by_name(&ctx.db.pool, sname).await?;
        if let Some(card) = cc {
            Ok(TransactionSource::CreditCard {
                credit_card_id: card.id,
            })
        } else {
            Err(ImportError::Handler(format!(
                "Cartão não encontrado: {}",
                sname
            )))
        }
    } else {
        Err(ImportError::Handler(format!(
            "Tipo de origem inválido: {}",
            stype
        )))
    }
}
