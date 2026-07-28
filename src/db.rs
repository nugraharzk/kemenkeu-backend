use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

pub async fn init_pool() -> MySqlPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:@127.0.0.1:3306/kemenkeu".to_string());

    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("cannot connect to database")
}

pub async fn run_migrations(pool: &MySqlPool) {
    let sql = include_str!("../migrations/001_init.sql");
    for statement in sql.split(';') {
        let s = statement.trim();
        if !s.is_empty() {
            match sqlx::query(s).execute(pool).await {
                Ok(_) => {}
                Err(e) => tracing::warn!("migration stmt skipped: {e}"),
            }
        }
    }
}