use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn connect_database() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(0)
        .idle_timeout(Duration::from_secs(300))
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .expect("Failed to connect to Neon database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database connected and migrations applied");

    pool
}
