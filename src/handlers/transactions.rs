use crate::error::AppError;
use crate::models::{CreateTxRequest, Transaction, TxQuery};
use axum::extract::{Path, Query, State};
use axum::http::header::COOKIE;
use axum::Json;
use sqlx::MySqlPool;

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

async fn is_admin(pool: &MySqlPool, user_id: i32) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar::<_, bool>("SELECT name = 'Admin' FROM users WHERE id = ?")
        .bind(user_id).fetch_one(pool).await?)
}

pub async fn list(
    State(pool): State<MySqlPool>,
    headers: axum::http::HeaderMap,
    Query(q): Query<TxQuery>,
) -> Result<Json<Vec<Transaction>>, AppError> {
    let user_id = session_id(&headers)?;
    let admin = is_admin(&pool, user_id).await?;
    let limit = q.limit.unwrap_or(100).min(200);
    let offset = q.offset.unwrap_or(0);
    let mut sql = "SELECT id, user_id, amount_cents, category_id, note, date, created_at FROM transactions WHERE 1=1".to_string();
    if !admin { sql.push_str(" AND user_id = ?"); }
    if let Some(cat) = q.category_id { sql.push_str(" AND category_id = ?"); }
    if let Some(ref month) = q.month { sql.push_str(" AND DATE_FORMAT(date, '%Y-%m') = ?"); }
    sql.push_str(" ORDER BY date DESC, id DESC LIMIT ? OFFSET ?");
    let mut query = sqlx::query_as::<_, Transaction>(&sql);
    if !admin { query = query.bind(user_id); }
    if let Some(cat) = q.category_id { query = query.bind(cat); }
    if let Some(ref month) = q.month { query = query.bind(month); }
    let txs = query.bind(limit).bind(offset).fetch_all(&pool).await?;
    Ok(Json(txs))
}

pub async fn create(
    State(pool): State<MySqlPool>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateTxRequest>,
) -> Result<Json<Transaction>, AppError> {
    let user_id = session_id(&headers)?;
    if req.amount_cents == 0 { return Err(AppError::BadRequest("amount must be non-zero".into())); }
    let result = sqlx::query("INSERT INTO transactions (user_id, amount_cents, category_id, note, date) VALUES (?, ?, ?, ?, ?)")
        .bind(user_id).bind(req.amount_cents).bind(req.category_id).bind(&req.note).bind(&req.date)
        .execute(&pool).await?;
    let tx = sqlx::query_as::<_, Transaction>("SELECT id, user_id, amount_cents, category_id, note, date, created_at FROM transactions WHERE id = ?")
        .bind(result.last_insert_id() as i32).fetch_one(&pool).await?;
    Ok(Json(tx))
}

pub async fn update(State(pool): State<MySqlPool>, headers: axum::http::HeaderMap, Path(id): Path<i32>, Json(req): Json<crate::models::UpdateTxRequest>) -> Result<Json<Transaction>, AppError> {
    let user_id = session_id(&headers)?;
    let admin = is_admin(&pool, user_id).await?;
    let existing = sqlx::query_as::<_, Transaction>("SELECT id, user_id, amount_cents, category_id, note, date, created_at FROM transactions WHERE id = ?")
        .bind(id).fetch_optional(&pool).await?.ok_or_else(|| AppError::NotFound("transaction not found".into()))?;
    if !admin && existing.user_id != user_id { return Err(AppError::Unauthorized("not allowed".into())); }
    let amount = req.amount_cents.unwrap_or(existing.amount_cents);
    let cat = req.category_id.unwrap_or(existing.category_id);
    let note = req.note.unwrap_or(existing.note);
    let date = req.date.unwrap_or(existing.date);
    sqlx::query("UPDATE transactions SET amount_cents=?, category_id=?, note=?, date=? WHERE id=?")
        .bind(amount).bind(cat).bind(&note).bind(&date).bind(id).execute(&pool).await?;
    Ok(Json(sqlx::query_as::<_, Transaction>("SELECT id, user_id, amount_cents, category_id, note, date, created_at FROM transactions WHERE id = ?").bind(id).fetch_one(&pool).await?))
}

pub async fn delete(State(pool): State<MySqlPool>, headers: axum::http::HeaderMap, Path(id): Path<i32>) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = session_id(&headers)?;
    let admin = is_admin(&pool, user_id).await?;
    let result = if admin { sqlx::query("DELETE FROM transactions WHERE id = ?").bind(id).execute(&pool).await? } else { sqlx::query("DELETE FROM transactions WHERE id = ? AND user_id = ?").bind(id).bind(user_id).execute(&pool).await? };
    if result.rows_affected() == 0 { return Err(AppError::NotFound("transaction not found".into())); }
    Ok(Json(serde_json::json!({"deleted": true})))
}
