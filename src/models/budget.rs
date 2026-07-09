use super::types::{BudgetPeriod, PositiveAmount};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Budget {
    pub id: i32,
    pub category_id: Option<i32>,
    pub tag_id: Option<i32>,
    pub amount_limit: PositiveAmount,
    pub period: BudgetPeriod,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BudgetWithSpend {
    #[serde(flatten)]
    pub budget: Budget,
    pub current_spend: Decimal,
}

impl Budget {
    pub async fn insert(
        pool: &sqlx::PgPool,
        category_id: Option<i32>,
        tag_id: Option<i32>,
        amount_limit: PositiveAmount,
        period: BudgetPeriod,
    ) -> Result<Self, sqlx::Error> {
        if category_id.is_none() && tag_id.is_none() {
            return Err(sqlx::Error::Protocol(
                "Budget must have category or tag.".into(),
            ));
        }

        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO budgets (category_id, tag_id, amount_limit, period)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(category_id)
        .bind(tag_id)
        .bind(amount_limit.as_decimal())
        .bind(period)
        .fetch_one(pool)
        .await
    }

    pub async fn list_with_spend(
        pool: &sqlx::PgPool,
        ref_date: NaiveDate,
    ) -> Result<Vec<BudgetWithSpend>, sqlx::Error> {
        let budgets = sqlx::query_as::<_, Self>("SELECT * FROM budgets ORDER BY id")
            .fetch_all(pool)
            .await?;

        let mut results = Vec::new();

        for budget in budgets {
            let spend = budget.current_spend(pool, ref_date).await?;
            results.push(BudgetWithSpend {
                budget,
                current_spend: spend,
            });
        }

        Ok(results)
    }

    pub async fn current_spend(
        &self,
        pool: &sqlx::PgPool,
        ref_date: NaiveDate,
    ) -> Result<Decimal, sqlx::Error> {
        let (start, end) = self.period_bounds(ref_date);

        let sum: Option<Decimal> = if let Some(cat_id) = self.category_id {
            sqlx::query_scalar(
                r#"
                SELECT SUM(
                    CASE 
                        WHEN transaction_type = 'income' THEN -amount 
                        ELSE amount 
                    END
                )
                FROM transactions
                WHERE category_id = $1 AND date >= $2 AND date <= $3 AND status = 'cleared'
                "#,
            )
            .bind(cat_id)
            .bind(start)
            .bind(end)
            .fetch_one(pool)
            .await?
        } else if let Some(tag_id) = self.tag_id {
            sqlx::query_scalar(
                r#"
                SELECT SUM(
                    CASE 
                        WHEN t.transaction_type = 'income' THEN -t.amount 
                        ELSE t.amount 
                    END
                )
                FROM transactions t
                INNER JOIN transaction_tags tt ON tt.transaction_id = t.id
                WHERE tt.tag_id = $1 AND t.date >= $2 AND t.date <= $3 AND t.status = 'cleared'
                "#,
            )
            .bind(tag_id)
            .bind(start)
            .bind(end)
            .fetch_one(pool)
            .await?
        } else {
            None
        };

        Ok(sum.unwrap_or(Decimal::ZERO))
    }

    pub async fn delete(pool: &sqlx::PgPool, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM budgets WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    fn period_bounds(&self, date: NaiveDate) -> (NaiveDate, NaiveDate) {
        match self.period {
            BudgetPeriod::Weekly => {
                let days_from_monday = date.weekday().num_days_from_monday();
                let start = date - chrono::Duration::days(days_from_monday as i64);
                let end = start + chrono::Duration::days(6);
                (start, end)
            }
            BudgetPeriod::Monthly => {
                let start = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap();
                let next_month = if date.month() == 12 {
                    NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap()
                };
                let end = next_month - chrono::Duration::days(1);
                (start, end)
            }
            BudgetPeriod::Yearly => {
                let start = NaiveDate::from_ymd_opt(date.year(), 1, 1).unwrap();
                let end = NaiveDate::from_ymd_opt(date.year(), 12, 31).unwrap();
                (start, end)
            }
        }
    }
}
