use crate::error::AppError;
use crate::models::{CreateUserRequest, UpdateUserRequest, User};
use axum::extract::{Path, State};
use axum::Json;
use sqlx::MySqlPool;

pub async fn list(State(pool): State<MySqlPool>) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as::<_, User>("SELECT id, name, monthly_budget_cents FROM users ORDER BY id")
        .fetch_all(&pool)
        .await?;
    Ok(Json(users))
}

pub async fn create(
    State(pool): State<MySqlPool>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, AppError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }

    let result = sqlx::query("INSERT INTO users (name, monthly_budget_cents) VALUES (?, ?)")
        .bind(&name)
        .bind(req.monthly_budget_cents)
        .execute(&pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("Duplicate") {
                AppError::BadRequest("user already exists".into())
            } else {
                AppError::Internal(e.to_string())
            }
        })?;

    let id = result.last_insert_id() as i32;
    let user = sqlx::query_as::<_, User>("SELECT id, name, monthly_budget_cents FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;

    Ok(Json(user))
}

pub async fn update(
    State(pool): State<MySqlPool>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, AppError> {
    let existing = sqlx::query_as::<_, User>("SELECT id, name, monthly_budget_cents FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;

    let name = req.name.unwrap_or(existing.name);
    let name = name.trim().to_string();
    let monthly_budget = req.monthly_budget_cents.unwrap_or(existing.monthly_budget_cents);
    if name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }

    sqlx::query("UPDATE users SET name = ?, monthly_budget_cents = ? WHERE id = ?")
        .bind(&name)
        .bind(monthly_budget)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("Duplicate") {
                AppError::BadRequest("user name already exists".into())
            } else {
                AppError::Internal(e.to_string())
            }
        })?;

    let user = sqlx::query_as::<_, User>("SELECT id, name, monthly_budget_cents FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;

    Ok(Json(user))
}

pub async fn delete(
    State(pool): State<MySqlPool>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let affected = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound("user not found".into()));
    }
    Ok(Json(serde_json::json!({"deleted": true})))
}
