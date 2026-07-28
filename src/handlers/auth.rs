use crate::error::AppError;
use crate::models::User;
use axum::extract::State;
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::HeaderValue;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::MySqlPool;

const SESSION_COOKIE: &str = "kemenkeu_session";
const MAX_AGE: &str = "604800"; // 7 days in seconds

#[derive(Deserialize)]
pub struct LoginRequest {
    pub user_id: i32,
}

pub async fn login(
    State(pool): State<MySqlPool>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT id, name, monthly_budget_cents FROM users WHERE id = ?")
        .bind(req.user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;

    let cookie = format!(
        "{SESSION_COOKIE}={}; Path=/; Max-Age={MAX_AGE}; SameSite=Lax",
        user.id
    );

    let mut response = Json(user).into_response();
    response
        .headers_mut()
        .insert(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());

    Ok(response)
}

fn extract_session_id(headers: &axum::http::HeaderMap) -> Option<i32> {
    let cookie_header = headers.get(COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .find_map(|pair| {
            let mut parts = pair.trim().splitn(2, '=');
            let key = parts.next()?.trim();
            let val = parts.next()?.trim();
            if key == SESSION_COOKIE {
                val.parse::<i32>().ok()
            } else {
                None
            }
        })
}

pub async fn me(
    State(pool): State<MySqlPool>,
    headers: axum::http::HeaderMap,
) -> Result<Json<User>, AppError> {
    let user_id = extract_session_id(&headers)
        .ok_or_else(|| AppError::Unauthorized("not logged in".into()))?;

    let user = sqlx::query_as::<_, User>("SELECT id, name, monthly_budget_cents FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;

    Ok(Json(user))
}

pub async fn logout() -> Result<impl IntoResponse, AppError> {
    let cookie = format!(
        "{SESSION_COOKIE}=; Path=/; Max-Age=0; SameSite=Lax"
    );

    let mut response = axum::http::Response::builder()
        .status(200)
        .body(String::from("ok"))
        .unwrap();
    response
        .headers_mut()
        .insert(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());

    Ok(response)
}
