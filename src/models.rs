use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub monthly_budget_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    #[serde(default)]
    pub monthly_budget_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub monthly_budget_cents: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: i32,
    pub name: String,
    #[serde(rename = "type")]
    pub category_type: String,
    pub icon: String,
    pub budget_percent: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: i32,
    pub user_id: i32,
    pub amount_cents: i64,
    pub category_id: i32,
    pub note: String,
    pub date: NaiveDate,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTxRequest {
    pub amount_cents: i64,
    pub category_id: i32,
    #[serde(default)]
    pub note: String,
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTxRequest {
    pub amount_cents: Option<i64>,
    pub category_id: Option<i32>,
    pub note: Option<String>,
    pub date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Budget {
    pub id: i32,
    pub category_id: i32,
    pub person: Option<String>,
    pub month: NaiveDate,
    pub amount_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct BudgetRequest {
    pub category_id: i32,
    pub person: Option<String>,
    pub month: NaiveDate,
    pub amount_cents: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SummaryRow {
    pub user_id: i32,
    pub category_id: i32,
    pub total_cents: i64,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct SummaryResponse {
    pub by_person_category: Vec<SummaryRow>,
    pub total_income: i64,
    pub total_expense: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MonthlyTrend {
    pub month: String,
    pub income: i64,
    pub expense: i64,
}

#[derive(Debug, Serialize)]
pub struct BudgetStatus {
    pub id: i32,
    pub category_id: i32,
    pub category_name: String,
    pub icon: String,
    pub person: Option<String>,
    pub budget_percent: i32,
    pub monthly_income: i64,
    pub budgeted: i64,
    pub spent: i64,
    pub remaining: i64,
}

#[derive(Debug, Deserialize)]
pub struct TxQuery {
    pub person: Option<String>,
    pub category_id: Option<i32>,
    pub month: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}