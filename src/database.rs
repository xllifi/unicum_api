use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn connect(postgres_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(postgres_url)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            username TEXT NOT NULL PRIMARY KEY,
            password_hash TEXT NOT NULL,
            permissions INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
