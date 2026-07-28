use crate::error::AppError;
use crate::models::{MonthlyTrend, SummaryResponse, SummaryRow};
use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Deserialize)]
pub struct SummaryQuery {
    pub month: Option<String>,
}

pub async fn get_summary(
    State(pool): State<MySqlPool>,
    Query(q): Query<SummaryQuery>,
) -> Result<Json<SummaryResponse>, AppError> {
    let month_filter = q.month.clone();

    let mut by_person_category = Vec::new();
    let mut total_income = 0i64;
    let mut total_expense = 0i64;

    let sql = if month_filter.is_some() {
        "SELECT person, category_id, CAST(SUM(amount_cents) AS SIGNED) as total_cents, COUNT(*) as count \
         FROM transactions WHERE DATE_FORMAT(date, '%Y-%m') = ? \
         GROUP BY person, category_id"
    } else {
        "SELECT person, category_id, CAST(SUM(amount_cents) AS SIGNED) as total_cents, COUNT(*) as count \
         FROM transactions GROUP BY person, category_id"
    };

    let mut query = sqlx::query_as::<_, SummaryRow>(sql);
    if let Some(ref m) = month_filter {
        query = query.bind(m);
    }

    let rows: Vec<SummaryRow> = query.fetch_all(&pool).await?;

    for row in &rows {
        if row.total_cents > 0 {
            total_income += row.total_cents;
        } else {
            total_expense += row.total_cents.abs();
        }
        by_person_category.push(SummaryRow {
            person: row.person.clone(),
            category_id: row.category_id,
            total_cents: row.total_cents,
            count: row.count,
        });
    }

    Ok(Json(SummaryResponse {
        by_person_category,
        total_income,
        total_expense,
    }))
}

pub async fn monthly_trends(
    State(pool): State<MySqlPool>,
) -> Result<Json<Vec<MonthlyTrend>>, AppError> {
    let rows = sqlx::query_as::<_, MonthlyTrend>(
        "SELECT DATE_FORMAT(date, '%Y-%m') as month, \
         CAST(SUM(CASE WHEN amount_cents > 0 THEN amount_cents ELSE 0 END) AS SIGNED) as income, \
         CAST(SUM(CASE WHEN amount_cents < 0 THEN ABS(amount_cents) ELSE 0 END) AS SIGNED) as expense \
         FROM transactions WHERE date >= DATE_SUB(CURRENT_DATE, INTERVAL 12 MONTH) \
         GROUP BY DATE_FORMAT(date, '%Y-%m') ORDER BY month",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(rows))
}