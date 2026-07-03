use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

// Parse, don't Validade
// Creates this boilerplate, but I find it worth the work

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)] // Avoid serde to create an array when parsing this tuple to JSON
pub struct PositiveAmount(Decimal);

impl PositiveAmount {
    pub fn inner(&self) -> Decimal {
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

impl sqlx::Decode<'_, sqlx::Postgres> for PositiveAmount {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let dec = <Decimal as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(PositiveAmount(dec))
    }
}

impl sqlx::Type<sqlx::Postgres> for PositiveAmount {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <Decimal as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct NonNegativeAmount(Decimal);

impl NonNegativeAmount {
    pub fn inner(&self) -> Decimal {
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

impl sqlx::Decode<'_, sqlx::Postgres> for NonNegativeAmount {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let dec = <Decimal as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(NonNegativeAmount(dec))
    }
}

impl sqlx::Type<sqlx::Postgres> for NonNegativeAmount {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <Decimal as sqlx::Type<sqlx::Postgres>>::type_info()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CategoryType {
    Income,
    Expense,
}

impl sqlx::Type<sqlx::Postgres> for CategoryType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for CategoryType {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "income" => Ok(CategoryType::Income),
            "expense" => Ok(CategoryType::Expense),
            _ => Err("Invalid category type".into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
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

impl sqlx::Decode<'_, sqlx::Postgres> for NonEmptyString {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(NonEmptyString(s))
    }
}

impl sqlx::Type<sqlx::Postgres> for NonEmptyString {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl NonEmptyString {
    pub fn inner(&self) -> &str {
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
