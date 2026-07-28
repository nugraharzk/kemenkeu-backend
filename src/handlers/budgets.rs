use crate::error::AppError;
use crate::models::{Budget, BudgetRequest, BudgetStatus, Category};
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Deserialize)]
pub struct BudgetQuery {
    pub month: Option<String>,
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
    Query(q): Query<StatusQuery>,
) -> Result<Json<Vec<BudgetStatus>>, AppError> {
    let month = q.month.unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m").to_string()
    });

    let budgets = sqlx::query_as::<_, Budget>(
        "SELECT id, category_id, person, month, amount_cents FROM budgets WHERE DATE_FORMAT(month, '%Y-%m') = ?",
    )
    .bind(&month)
    .fetch_all(&pool)
    .await?;

    let mut statuses = Vec::new();

    for b in budgets {
        let spent: i64 = if let Some(ref person) = b.person {
            sqlx::query_scalar::<_, Option<i64>>(
                "SELECT CAST(COALESCE(SUM(amount_cents), 0) AS SIGNED) FROM transactions \
                 WHERE DATE_FORMAT(date, '%Y-%m') = ? AND category_id = ? AND person = ? AND amount_cents < 0",
            )
            .bind(&month)
            .bind(b.category_id)
            .bind(person)
            .fetch_one(&pool)
            .await?
            .unwrap_or(0)
            .abs()
        } else {
            sqlx::query_scalar::<_, Option<i64>>(
                "SELECT CAST(COALESCE(SUM(amount_cents), 0) AS SIGNED) FROM transactions \
                 WHERE DATE_FORMAT(date, '%Y-%m') = ? AND category_id = ? AND amount_cents < 0",
            )
            .bind(&month)
            .bind(b.category_id)
            .fetch_one(&pool)
            .await?
            .unwrap_or(0)
            .abs()
        };

        let cat = sqlx::query_as::<_, Category>(
            "SELECT id, name, type AS category_type, icon FROM categories WHERE id = ?",
        )
        .bind(b.category_id)
        .fetch_one(&pool)
        .await
        .unwrap_or(Category {
            id: b.category_id,
            name: "Unknown".into(),
            category_type: "expense".into(),
            icon: "❓".into(),
        });

        statuses.push(BudgetStatus {
            id: b.id,
            category_id: b.category_id,
            category_name: cat.name,
            icon: cat.icon,
            person: b.person,
            budgeted: b.amount_cents,
            spent,
            remaining: b.amount_cents - spent,
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