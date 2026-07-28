use crate::error::AppError;
use crate::models::{Budget, BudgetRequest, BudgetStatus, Category};
use axum::extract::{Path, Query, State};
use axum::http::header::COOKIE;
use axum::Json;
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Deserialize)]
pub struct BudgetQuery {
    pub month: Option<String>,
}

fn session_id(headers: &axum::http::HeaderMap) -> Result<i32, AppError> {
    headers
        .get(COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find_map(|p| {
            let mut pair = p.trim().splitn(2, '=');
            if pair.next()? == "kemenkeu_session" { pair.next()?.parse().ok() } else { None }
        }))
        .ok_or_else(|| AppError::Unauthorized("not logged in".into()))
}

pub async fn list(
    State(pool): State<MySqlPool>,
    Query(q): Query<BudgetQuery>,
) -> Result<Json<Vec<Budget>>, AppError> {
    let mut sql = "SELECT id, category_id, person, month, amount_cents FROM budgets WHERE 1=1".to_string();
    if let Some(ref month) = q.month {
        sql.push_str(" AND DATE_FORMAT(month, '%Y-%m') = ?");
    }
    let mut query = sqlx::query_as::<_, Budget>(&sql);
    if let Some(ref m) = q.month {
        query = query.bind(m);
    }
    let budgets = query.fetch_all(&pool).await?;
    Ok(Json(budgets))
}

pub async fn upsert(
    State(pool): State<MySqlPool>,
    Json(req): Json<BudgetRequest>,
) -> Result<Json<Budget>, AppError> {
    if req.amount_cents < 0 {
        return Err(AppError::BadRequest("budget amount must be >= 0".into()));
    }
    sqlx::query(
        "INSERT INTO budgets (category_id, person, month, amount_cents) \
         VALUES (?, ?, ?, ?) \
         ON DUPLICATE KEY UPDATE amount_cents = VALUES(amount_cents)",
    )
    .bind(req.category_id)
    .bind(&req.person)
    .bind(&req.month)
    .bind(req.amount_cents)
    .execute(&pool)
    .await?;

    let budget = sqlx::query_as::<_, Budget>(
        "SELECT id, category_id, person, month, amount_cents FROM budgets \
         WHERE category_id = ? AND person <=> ? AND month = ?",
    )
    .bind(req.category_id)
    .bind(&req.person)
    .bind(&req.month)
    .fetch_one(&pool)
    .await?;

    Ok(Json(budget))
}

#[derive(Deserialize)]
pub struct StatusQuery {
    pub month: Option<String>,
}

pub async fn status(
    State(pool): State<MySqlPool>,
    headers: axum::http::HeaderMap,
    Query(q): Query<StatusQuery>,
) -> Result<Json<Vec<BudgetStatus>>, AppError> {
    let user_id = session_id(&headers)?;
    let month = q.month.unwrap_or_else(|| chrono::Local::now().format("%Y-%m").to_string());

    let monthly_income = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT monthly_budget_cents FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await?
    .unwrap_or(0);

    let categories = sqlx::query_as::<_, Category>(
        "SELECT id, name, type AS category_type, icon, budget_percent FROM categories WHERE type = 'expense' ORDER BY id",
    )
    .fetch_all(&pool)
    .await?;

    let mut statuses = Vec::with_capacity(categories.len());
    for cat in categories {
        let budgeted = (monthly_income * cat.budget_percent as i64) / 100;

        let spent = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT CAST(COALESCE(SUM(amount_cents), 0) AS SIGNED) FROM transactions \
             WHERE DATE_FORMAT(date, '%Y-%m') = ? AND category_id = ? AND user_id = ? AND amount_cents < 0",
        )
        .bind(&month)
        .bind(cat.id)
        .bind(user_id)
        .fetch_one(&pool)
        .await?
        .unwrap_or(0)
        .abs();

        statuses.push(BudgetStatus {
            id: cat.id,
            category_id: cat.id,
            category_name: cat.name,
            icon: cat.icon,
            person: None,
            budget_percent: cat.budget_percent,
            monthly_income,
            budgeted,
            spent,
            remaining: budgeted - spent,
        });
    }

    Ok(Json(statuses))
}

pub async fn delete(
    State(pool): State<MySqlPool>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let affected = sqlx::query("DELETE FROM budgets WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound("budget not found".into()));
    }
    Ok(Json(serde_json::json!({"deleted": true})))
}