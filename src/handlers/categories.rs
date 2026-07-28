use crate::error::AppError;
use crate::models::Category;
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Deserialize, Default)]
pub struct CategoryQuery {
    #[serde(rename = "type")]
    pub category_type: Option<String>,
}

pub async fn list(
    State(pool): State<MySqlPool>,
    Query(q): Query<CategoryQuery>,
) -> Result<Json<Vec<Category>>, AppError> {
    let mut sql = "SELECT id, name, type AS category_type, icon FROM categories".to_string();
    if let Some(ref t) = q.category_type {
        sql.push_str(" WHERE type = ?");
    }
    sql.push_str(" ORDER BY type, id");

    let mut query = sqlx::query_as::<_, Category>(&sql);
    if let Some(ref t) = q.category_type {
        query = query.bind(t);
    }

    let cats = query.fetch_all(&pool).await?;
    Ok(Json(cats))
}

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub category_type: String,
    #[serde(default = "default_icon")]
    pub icon: String,
}

fn default_icon() -> String {
    "📦".into()
}

pub async fn create(
    State(pool): State<MySqlPool>,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<Json<Category>, AppError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if req.category_type != "income" && req.category_type != "expense" {
        return Err(AppError::BadRequest("type must be income or expense".into()));
    }

    let result = sqlx::query("INSERT INTO categories (name, type, icon) VALUES (?, ?, ?)")
        .bind(&name)
        .bind(&req.category_type)
        .bind(&req.icon)
        .execute(&pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("Duplicate") {
                AppError::BadRequest("category already exists".into())
            } else {
                AppError::Internal(e.to_string())
            }
        })?;

    let id = result.last_insert_id() as i32;
    let cat = sqlx::query_as::<_, Category>(
        "SELECT id, name, type AS category_type, icon FROM categories WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(cat))
}

#[derive(Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub category_type: Option<String>,
    pub icon: Option<String>,
}

pub async fn update(
    State(pool): State<MySqlPool>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCategoryRequest>,
) -> Result<Json<Category>, AppError> {
    let existing =
        sqlx::query_as::<_, Category>("SELECT id, name, type AS category_type, icon FROM categories WHERE id = ?")
            .bind(id)
            .fetch_optional(&pool)
            .await?
            .ok_or_else(|| AppError::NotFound("category not found".into()))?;

    let name = req.name.unwrap_or(existing.name);
    let name = name.trim().to_string();
    let cat_type = req.category_type.unwrap_or(existing.category_type);
    let icon = req.icon.unwrap_or(existing.icon);

    if name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if cat_type != "income" && cat_type != "expense" {
        return Err(AppError::BadRequest("type must be income or expense".into()));
    }

    sqlx::query("UPDATE categories SET name = ?, type = ?, icon = ? WHERE id = ?")
        .bind(&name)
        .bind(&cat_type)
        .bind(&icon)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("Duplicate") {
                AppError::BadRequest("category already exists".into())
            } else {
                AppError::Internal(e.to_string())
            }
        })?;

    let cat = sqlx::query_as::<_, Category>(
        "SELECT id, name, type AS category_type, icon FROM categories WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(cat))
}

pub async fn delete(
    State(pool): State<MySqlPool>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let affected = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound("category not found".into()));
    }
    Ok(Json(serde_json::json!({"deleted": true})))
}
