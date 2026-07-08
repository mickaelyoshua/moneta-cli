use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

// Parse, don't Validade
// Creates this boilerplate, but I find it worth the work

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct DayOfMonth(i16);

impl std::str::FromStr for DayOfMonth {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: i16 = s.parse().map_err(|_| "Número inválido")?;
        if (1..=28).contains(&val) {
            Ok(DayOfMonth(val))
        } else {
            Err("Dia deve ser entre 1 e 28 (por segurança de fevereiro)")
        }
    }
}

impl DayOfMonth {
    pub fn as_i16(&self) -> i16 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct PositiveAmount(Decimal);

impl PositiveAmount {
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }
}

impl std::str::FromStr for PositiveAmount {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dec = Decimal::from_str(s).map_err(|_| "Formato numérico inválido")?;
        if dec > Decimal::ZERO {
            Ok(PositiveAmount(dec))
        } else {
            Err("Valor deve ser maior que zero")
        }
    }
}

impl<'de> Deserialize<'de> for PositiveAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dec = <Decimal as Deserialize>::deserialize(deserializer)?;
        if dec > Decimal::ZERO {
            Ok(PositiveAmount(dec))
        } else {
            Err(serde::de::Error::custom("Valor deve ser > 0"))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct NonNegativeAmount(Decimal);

impl NonNegativeAmount {
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }
}

impl<'de> Deserialize<'de> for NonNegativeAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dec = <Decimal as Deserialize>::deserialize(deserializer)?;
        if dec >= Decimal::ZERO {
            Ok(NonNegativeAmount(dec))
        } else {
            Err(serde::de::Error::custom("Valor não pode ser negativo"))
        }
    }
}

impl std::str::FromStr for NonNegativeAmount {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dec = Decimal::from_str(s).map_err(|_| "Formato numérico inválido")?;
        if dec >= Decimal::ZERO {
            Ok(NonNegativeAmount(dec))
        } else {
            Err("Valor não pode ser negativo")
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum, sqlx::Type,
)]
#[sqlx(type_name = "transaction_type_enum", rename_all = "lowercase")]
pub enum TransactionType {
    Income,
    Expense,
    Transfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_status_enum", rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Cleared,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum, sqlx::Type,
)]
#[sqlx(type_name = "invoice_status_enum", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum InvoiceStatus {
    Open,
    Closed,
    Paid,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, clap::ValueEnum,
)]
#[sqlx(type_name = "account_type_enum", rename_all = "lowercase")]
pub enum AccountType {
    Checking,
    Savings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "recurrence_frequency_enum", rename_all = "lowercase")]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum, sqlx::Type,
)]
#[sqlx(type_name = "category_type_enum", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CategoryType {
    Income,
    Expense,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct NonEmptyString(String);

impl<'de> Deserialize<'de> for NonEmptyString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let trimmed = s.trim();
        if trimmed.is_empty() {
            Err(serde::de::Error::custom("A string não pode ser vazia"))
        } else {
            Ok(NonEmptyString(trimmed.to_string()))
        }
    }
}

impl NonEmptyString {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::str::FromStr for NonEmptyString {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            Err("A string não pode ser vazia ou conter apenas espaços")
        } else {
            Ok(NonEmptyString(trimmed.to_string()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransactionSource {
    Account { account_id: i32 },
    CreditCard { credit_card_id: i32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_non_empty_string_valid() {
        assert!(NonEmptyString::from_str("valid").is_ok());
        assert!(NonEmptyString::from_str("  valid  ").is_ok()); // Trims or accepts? Current implementation might just check empty, but let's see.
    }

    #[test]
    fn test_non_empty_string_invalid() {
        assert!(NonEmptyString::from_str("").is_err());
        assert!(NonEmptyString::from_str("   ").is_err());
    }

    #[test]
    fn test_positive_amount_valid() {
        assert!(PositiveAmount::from_str("100.50").is_ok());
    }

    #[test]
    fn test_positive_amount_invalid() {
        assert!(PositiveAmount::from_str("0").is_err());
        assert!(PositiveAmount::from_str("-100.50").is_err());
        assert!(PositiveAmount::from_str("abc").is_err());
    }

    #[test]
    fn test_non_negative_amount_valid() {
        assert!(NonNegativeAmount::from_str("0").is_ok());
        assert!(NonNegativeAmount::from_str("100.50").is_ok());
    }

    #[test]
    fn test_non_negative_amount_invalid() {
        assert!(NonNegativeAmount::from_str("-100.50").is_err());
        assert!(NonNegativeAmount::from_str("abc").is_err());
    }

    #[test]
    fn test_day_of_month_valid() {
        assert!(DayOfMonth::from_str("1").is_ok());
        assert!(DayOfMonth::from_str("15").is_ok());
        assert!(DayOfMonth::from_str("28").is_ok());
    }

    #[test]
    fn test_day_of_month_invalid() {
        assert!(DayOfMonth::from_str("0").is_err());
        assert!(DayOfMonth::from_str("29").is_err());
        assert!(DayOfMonth::from_str("31").is_err());
        assert!(DayOfMonth::from_str("-5").is_err());
        assert!(DayOfMonth::from_str("abc").is_err());
    }
}
