use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn try_connect_database() -> Result<PgPool, String> {
    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set".to_string())?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(0)
        .idle_timeout(Duration::from_secs(300))
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(pool)
}
