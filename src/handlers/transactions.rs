use crate::error::AppError;
use crate::models::{
    CreateTxRequest, Transaction, TxQuery,
};
use axum::extract::{Path, Query, State};
use axum::Json;
use sqlx::MySqlPool;

pub async fn list(
    State(pool): State<MySqlPool>,
    Query(q): Query<TxQuery>,
) -> Result<Json<Vec<Transaction>>, AppError> {
    let limit = q.limit.unwrap_or(100).min(200);
    let offset = q.offset.unwrap_or(0);

    let mut sql = "SELECT id, person, amount_cents, category_id, note, date, created_at FROM transactions WHERE 1=1".to_string();

    if let Some(ref person) = q.person {
        sql.push_str(" AND person = ?");
    }
    if let Some(cat) = q.category_id {
        sql.push_str(" AND category_id = ?");
    }
    if let Some(ref month) = q.month {
        sql.push_str(" AND DATE_FORMAT(date, '%Y-%m') = ?");
    }
    sql.push_str(" ORDER BY date DESC, id DESC LIMIT ? OFFSET ?");

    let mut query = sqlx::query_as::<_, Transaction>(&sql);
    if let Some(ref person) = q.person {
        query = query.bind(person);
    }
    if let Some(cat) = q.category_id {
        query = query.bind(cat);
    }
    if let Some(ref month) = q.month {
        query = query.bind(month);
    }
    query = query.bind(limit).bind(offset);

    let txs = query.fetch_all(&pool).await?;
    Ok(Json(txs))
}

pub async fn create(
    State(pool): State<MySqlPool>,
    Json(req): Json<CreateTxRequest>,
) -> Result<Json<Transaction>, AppError> {
    if req.amount_cents == 0 {
        return Err(AppError::BadRequest("amount must be non-zero".into()));
    }
    if req.person != "Saya" && req.person != "Pasangan" {
        return Err(AppError::BadRequest("person must be Saya or Pasangan".into()));
    }

    let result = sqlx::query(
        "INSERT INTO transactions (person, amount_cents, category_id, note, date) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&req.person)
    .bind(req.amount_cents)
    .bind(req.category_id)
    .bind(&req.note)
    .bind(&req.date)
    .execute(&pool)
    .await?;

    let id = result.last_insert_id() as i32;
    let tx = sqlx::query_as::<_, Transaction>(
        "SELECT id, person, amount_cents, category_id, note, date, created_at FROM transactions WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(tx))
}

pub async fn update(
    State(pool): State<MySqlPool>,
    Path(id): Path<i32>,
    Json(req): Json<crate::models::UpdateTxRequest>,
) -> Result<Json<Transaction>, AppError> {
    let existing = sqlx::query_as::<_, Transaction>(
        "SELECT id, person, amount_cents, category_id, note, date, created_at FROM transactions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("transaction not found".into()))?;

    let person = req.person.unwrap_or(existing.person);
    let amount = req.amount_cents.unwrap_or(existing.amount_cents);
    let cat = req.category_id.unwrap_or(existing.category_id);
    let note = req.note.unwrap_or(existing.note);
    let date = req.date.unwrap_or(existing.date);

    sqlx::query(
        "UPDATE transactions SET person=?, amount_cents=?, category_id=?, note=?, date=? WHERE id=?",
    )
    .bind(&person)
    .bind(amount)
    .bind(cat)
    .bind(&note)
    .bind(&date)
    .bind(id)
    .execute(&pool)
    .await?;

    let tx = sqlx::query_as::<_, Transaction>(
        "SELECT id, person, amount_cents, category_id, note, date, created_at FROM transactions WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(tx))
}

pub async fn delete(
    State(pool): State<MySqlPool>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let affected = sqlx::query("DELETE FROM transactions WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound("transaction not found".into()));
    }
    Ok(Json(serde_json::json!({"deleted": true})))
}