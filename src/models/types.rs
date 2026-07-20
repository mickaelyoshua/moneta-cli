use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

// Parse, don't Validate
// Creates this boilerplate, but I find it worth the work

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct DayOfMonth(i16);

macro_rules! impl_i16_validated_newtype {
    ($name:ident, $validate:expr, $err_msg:expr) => {
        impl std::str::FromStr for $name {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let val: i16 = s.parse().map_err(|_| "Invalid number format")?;
                if $validate(val) {
                    Ok($name(val))
                } else {
                    Err($err_msg)
                }
            }
        }

        impl TryFrom<i16> for $name {
            type Error = &'static str;

            fn try_from(val: i16) -> Result<Self, Self::Error> {
                if $validate(val) {
                    Ok($name(val))
                } else {
                    Err($err_msg)
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let val = <i16 as serde::Deserialize>::deserialize(deserializer)?;
                if $validate(val) {
                    Ok($name(val))
                } else {
                    Err(serde::de::Error::custom($err_msg))
                }
            }
        }
    };
}

macro_rules! impl_decimal_validated_newtype {
    ($name:ident, $validate:expr, $err_msg:expr) => {
        impl std::str::FromStr for $name {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let dec =
                    rust_decimal::Decimal::from_str(s).map_err(|_| "Invalid number format")?;
                if $validate(dec) {
                    Ok($name(dec))
                } else {
                    Err($err_msg)
                }
            }
        }

        impl TryFrom<rust_decimal::Decimal> for $name {
            type Error = &'static str;

            fn try_from(dec: rust_decimal::Decimal) -> Result<Self, Self::Error> {
                if $validate(dec) {
                    Ok($name(dec))
                } else {
                    Err($err_msg)
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let dec = <rust_decimal::Decimal as serde::Deserialize>::deserialize(deserializer)?;
                if $validate(dec) {
                    Ok($name(dec))
                } else {
                    Err(serde::de::Error::custom($err_msg))
                }
            }
        }
    };
}

impl_i16_validated_newtype!(
    DayOfMonth,
    |v| (1..=28).contains(&v),
    "Day must be between 1 and 28 (limited by February)"
);

impl DayOfMonth {
    pub fn as_i16(&self) -> i16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct PositiveAmount(Decimal);

impl PositiveAmount {
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }
}

impl_decimal_validated_newtype!(
    PositiveAmount,
    |v| v > rust_decimal::Decimal::ZERO,
    "Value must be greater than zero"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct NonNegativeAmount(Decimal);

impl NonNegativeAmount {
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }
}

impl_decimal_validated_newtype!(
    NonNegativeAmount,
    |v| v >= rust_decimal::Decimal::ZERO,
    "Value cannot be negative"
);

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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum, sqlx::Type,
)]
#[sqlx(type_name = "budget_period", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BudgetPeriod {
    Weekly,
    Monthly,
    Yearly,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum, sqlx::Type,
)]
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
            Err(serde::de::Error::custom("String cannot be empty"))
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
            Err("String cannot be empty or have only spaces")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct InstallmentCount(i16);

impl InstallmentCount {
    pub fn get(&self) -> i16 {
        self.0
    }
}

impl_i16_validated_newtype!(
    InstallmentCount,
    |v| v > 0,
    "Installments number must be greater than zero"
);

impl std::fmt::Display for InstallmentCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct InstallmentNumber(i16);

impl InstallmentNumber {
    pub fn get(&self) -> i16 {
        self.0
    }
}

impl_i16_validated_newtype!(
    InstallmentNumber,
    |v| v > 0,
    "Installments number must be greater than zero"
);

impl std::fmt::Display for InstallmentNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct Month(i16);

impl Month {
    pub fn as_i16(&self) -> i16 {
        self.0
    }
}

impl_i16_validated_newtype!(
    Month,
    |v| (1..=12).contains(&v),
    "Month must be between 1 and 12"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct Year(i16);

impl Year {
    pub fn as_i16(&self) -> i16 {
        self.0
    }
}

impl_i16_validated_newtype!(
    Year,
    |v| v >= 2000,
    "Year must be greater than or equal to 2000"
);

pub fn safe_from_ymd(year: i32, month: u32, day: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap_or_else(|| {
        let mut nm = month + 1;
        let mut ny = year;
        if nm > 12 {
            nm = 1;
            ny += 1;
        }
        chrono::NaiveDate::from_ymd_opt(ny, nm, 1)
            .expect("Year and month should be valid to get the 1st day")
            - chrono::Duration::days(1)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_non_empty_string_valid() {
        assert!(NonEmptyString::from_str("valid").is_ok());
        assert!(NonEmptyString::from_str("  valid  ").is_ok());
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
