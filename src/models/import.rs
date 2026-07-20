use crate::models::transaction::Transaction;
use crate::models::types::{NonEmptyString, PositiveAmount};
use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct RawCsvRecord {
    pub date: String,
    pub amount: String,
    pub description: String,
    pub category: String,
    pub tags: String,
    pub installments: String,
    pub source_type: String,
    pub source_name: String,
}

#[derive(Debug, Clone)]
pub struct ParsedCsvRecord {
    pub date: NaiveDate,
    pub amount: PositiveAmount,
    pub description: NonEmptyString,
    pub category: NonEmptyString,
    pub tags: Vec<NonEmptyString>,
    pub installment: Option<(i16, i16)>,
    pub source_type: NonEmptyString,
    pub source_name: NonEmptyString,
}

impl TryFrom<RawCsvRecord> for ParsedCsvRecord {
    type Error = String;

    fn try_from(raw: RawCsvRecord) -> Result<Self, Self::Error> {
        let date = NaiveDate::from_str(&raw.date)
            .map_err(|_| format!("Invalid date: {}", raw.date))?;
        let amount = PositiveAmount::from_str(&raw.amount)
            .map_err(|_| format!("Invalid amount: {}", raw.amount))?;
        let description = NonEmptyString::from_str(&raw.description)
            .map_err(|_| format!("Empty description for: {}", raw.description))?;
        let category = NonEmptyString::from_str(&raw.category)
            .map_err(|_| format!("Empty category for: {}", raw.description))?;

        let tags = if raw.tags.trim().is_empty() {
            vec![]
        } else {
            raw.tags
                .split('|')
                .filter_map(|t| NonEmptyString::from_str(t.trim()).ok())
                .collect()
        };

        let installment = if raw.installments.trim().is_empty() {
            None
        } else {
            let parts: Vec<&str> = raw.installments.split('/').collect();
            if parts.len() == 2 {
                let current: i16 = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid current installment number: {}", parts[0]))?;
                let total: i16 = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid total installments number: {}", parts[1]))?;
                if current <= 0 || total <= 0 {
                    return Err(format!("Invalid installment values: {}", raw.installments));
                }
                Some((current, total))
            } else {
                return Err(format!(
                    "Invalid installment format (expected x/y): {}",
                    raw.installments
                ));
            }
        };

        let source_type = NonEmptyString::from_str(&raw.source_type)
            .map_err(|_| format!("Invalid source type: {}", raw.source_type))?;
        let source_name = NonEmptyString::from_str(&raw.source_name)
            .map_err(|_| format!("Empty source name: {}", raw.source_name))?;

        Ok(ParsedCsvRecord {
            date,
            amount,
            description,
            category,
            tags,
            installment,
            source_type,
            source_name,
        })
    }
}

pub fn deduplicate_records(
    csv_records: &[ParsedCsvRecord],
    db_transactions: &[Transaction],
) -> Vec<ParsedCsvRecord> {
    let mut csv_freq: HashMap<(NaiveDate, rust_decimal::Decimal, String), usize> = HashMap::new();
    for rec in csv_records {
        let key = (
            rec.date,
            rec.amount.as_decimal(),
            rec.description.as_str().to_string(),
        );
        *csv_freq.entry(key).or_insert(0) += 1;
    }

    let mut db_freq: HashMap<(NaiveDate, rust_decimal::Decimal, String), usize> = HashMap::new();
    for tx in db_transactions {
        let key = (
            tx.date,
            tx.amount.as_decimal(),
            tx.description.as_str().to_string(),
        );
        *db_freq.entry(key).or_insert(0) += 1;
    }

    let mut to_insert = Vec::new();
    for rec in csv_records {
        let key = (
            rec.date,
            rec.amount.as_decimal(),
            rec.description.as_str().to_string(),
        );
        let csv_count = *csv_freq.get(&key).unwrap_or(&0);
        let db_count = *db_freq.get(&key).unwrap_or(&0);

        if db_count >= csv_count {
            db_freq.entry(key.clone()).and_modify(|e| *e -= 1);
            continue;
        }

        db_freq
            .entry(key.clone())
            .and_modify(|e| *e += 1)
            .or_insert(1);
        to_insert.push(rec.clone());
    }

    to_insert
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::Transaction;
    use crate::models::types::{TransactionType, PositiveAmount, NonEmptyString};
    use chrono::Utc;

    fn mock_parsed_record(date: &str, amount: f64, description: &str) -> ParsedCsvRecord {
        ParsedCsvRecord {
            date: NaiveDate::from_str(date).unwrap(),
            amount: PositiveAmount::try_from(rust_decimal::Decimal::from_f64_retain(amount).unwrap()).unwrap(),
            description: NonEmptyString::from_str(description).unwrap(),
            category: NonEmptyString::from_str("Test").unwrap(),
            tags: vec![],
            installment: None,
            source_type: NonEmptyString::from_str("account").unwrap(),
            source_name: NonEmptyString::from_str("Test Account").unwrap(),
        }
    }

    fn mock_transaction(date: &str, amount: f64, description: &str) -> Transaction {
        Transaction {
            id: 1,
            source: crate::models::types::TransactionSource::Account { account_id: 1 },
            category_id: None,
            transaction_type: TransactionType::Expense,
            amount: PositiveAmount::try_from(rust_decimal::Decimal::from_f64_retain(amount).unwrap()).unwrap(),
            date: NaiveDate::from_str(date).unwrap(),
            description: NonEmptyString::from_str(description).unwrap(),
            status: crate::models::types::TransactionStatus::Cleared,
            installment_id: None,
            installment_number: None,
            invoice_id: None,
            recurrence_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec![],
        }
    }

    #[test]
    fn test_deduplicate_no_db_records() {
        let csv = vec![
            mock_parsed_record("2023-01-01", 10.50, "Uber"),
            mock_parsed_record("2023-01-02", 15.00, "Ifood"),
        ];
        let db = vec![];
        let result = deduplicate_records(&csv, &db);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_deduplicate_exact_match() {
        let csv = vec![
            mock_parsed_record("2023-01-01", 10.50, "Uber"),
            mock_parsed_record("2023-01-02", 15.00, "Ifood"),
        ];
        let db = vec![
            mock_transaction("2023-01-01", 10.50, "Uber"),
        ];
        let result = deduplicate_records(&csv, &db);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description.as_str(), "Ifood");
    }

    #[test]
    fn test_deduplicate_frequency_count() {
        let csv = vec![
            mock_parsed_record("2023-01-01", 10.50, "Uber"),
            mock_parsed_record("2023-01-01", 10.50, "Uber"),
        ];
        let db = vec![
            mock_transaction("2023-01-01", 10.50, "Uber"),
        ];
        let result = deduplicate_records(&csv, &db);
        assert_eq!(result.len(), 1, "Should insert the second Uber charge because DB only has 1");
    }

    #[test]
    fn test_deduplicate_different_amount() {
        let csv = vec![
            mock_parsed_record("2023-01-01", 11.50, "Uber"),
        ];
        let db = vec![
            mock_transaction("2023-01-01", 10.50, "Uber"),
        ];
        let result = deduplicate_records(&csv, &db);
        assert_eq!(result.len(), 1, "Different amount should not deduplicate");
    }
}
