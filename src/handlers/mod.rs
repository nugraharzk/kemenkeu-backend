mod auth;
mod budgets;
mod categories;
mod summary;
mod transactions;
mod users;

use axum::{routing, Router};
use sqlx::MySqlPool;

pub fn routes() -> Router<MySqlPool> {
    Router::new()
        .route("/api/auth/login", routing::post(auth::login))
        .route("/api/auth/me", routing::get(auth::me))
        .route("/api/auth/logout", routing::post(auth::logout))
        .route("/api/users", routing::get(users::list).post(users::create))
        .route(
            "/api/users/{id}",
            routing::put(users::update).delete(users::delete),
        )
        .route("/api/categories", routing::get(categories::list).post(categories::create))
        .route(
            "/api/categories/{id}",
            routing::put(categories::update).delete(categories::delete),
        )
        .route("/api/transactions", routing::get(transactions::list).post(transactions::create))
        .route(
            "/api/transactions/{id}",
            routing::put(transactions::update).delete(transactions::delete),
        )
        .route("/api/summary", routing::get(summary::get_summary))
        .route("/api/summary/trends", routing::get(summary::monthly_trends))
        .route("/api/budgets", routing::get(budgets::list).post(budgets::upsert))
        .route("/api/budgets/status", routing::get(budgets::status))
        .route(
            "/api/budgets/{id}",
            routing::delete(budgets::delete),
        )
}