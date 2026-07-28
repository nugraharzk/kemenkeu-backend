use crate::error::AppError;
use crate::models::{MonthlyTrend, SummaryResponse, SummaryRow};
use axum::extract::{Query, State};
use axum::http::header::COOKIE;
use axum::Json;
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Deserialize)]
pub struct SummaryQuery {
    pub month: Option<String>,
}

fn session_id(headers: &axum::http::HeaderMap) -> Result<i32, AppError> {
    headers.get(COOKIE).and_then(|v| v.to_str().ok()).and_then(|s| s.split(';').find_map(|p| {
        let mut pair = p.trim().splitn(2, '=');
        if pair.next()? == "kemenkeu_session" { pair.next()?.parse().ok() } else { None }
    })).ok_or_else(|| AppError::Unauthorized("not logged in".into()))
}

async fn is_admin(pool: &MySqlPool, user_id: i32) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar::<_, bool>("SELECT name = 'Admin' FROM users WHERE id = ?")
        .bind(user_id).fetch_one(pool).await?)
}

pub async fn get_summary(
    State(pool): State<MySqlPool>,
    headers: axum::http::HeaderMap,
    Query(q): Query<SummaryQuery>,
) -> Result<Json<SummaryResponse>, AppError> {
    let user_id = session_id(&headers)?;
    let admin = is_admin(&pool, user_id).await?;
    let month = q.month;
    let mut sql = "SELECT user_id, category_id, CAST(SUM(amount_cents) AS SIGNED) as total_cents, COUNT(*) as count FROM transactions WHERE 1=1".to_string();
    if !admin { sql.push_str(" AND user_id = ?"); }
    if month.is_some() { sql.push_str(" AND DATE_FORMAT(date, '%Y-%m') = ?"); }
    sql.push_str(" GROUP BY user_id, category_id");
    let mut query = sqlx::query_as::<_, SummaryRow>(&sql);
    if !admin { query = query.bind(user_id); }
    if let Some(ref m) = month { query = query.bind(m); }
    let rows = query.fetch_all(&pool).await?;
    let total_income = rows.iter().filter(|r| r.total_cents > 0).map(|r| r.total_cents).sum();
    let total_expense = rows.iter().filter(|r| r.total_cents < 0).map(|r| r.total_cents.abs()).sum();
    Ok(Json(SummaryResponse { by_person_category: rows, total_income, total_expense }))
}

pub async fn monthly_trends(State(pool): State<MySqlPool>) -> Result<Json<Vec<MonthlyTrend>>, AppError> {
    let rows = sqlx::query_as::<_, MonthlyTrend>(
        "SELECT DATE_FORMAT(date, '%Y-%m') as month, CAST(SUM(CASE WHEN amount_cents > 0 THEN amount_cents ELSE 0 END) AS SIGNED) as income, CAST(SUM(CASE WHEN amount_cents < 0 THEN ABS(amount_cents) ELSE 0 END) AS SIGNED) as expense FROM transactions WHERE date >= DATE_SUB(CURRENT_DATE, INTERVAL 12 MONTH) GROUP BY DATE_FORMAT(date, '%Y-%m') ORDER BY month",
    ).fetch_all(&pool).await?;
    Ok(Json(rows))
}
