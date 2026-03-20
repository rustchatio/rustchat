use std::path::PathBuf;

/// Load seed data SQL into the test database
///
/// # Usage
/// ```rust,ignore
/// let pool = setup_test_db().await;
/// load_seed_data(&pool).await.expect("Failed to load seed data");
/// ```
pub async fn load_seed_data(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let mut fixtures_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixtures_path.push("tests/fixtures/seed_data.sql");

    let seed_sql = std::fs::read_to_string(&fixtures_path)
        .expect("Failed to read seed_data.sql fixture");

    sqlx::query(&seed_sql)
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires test database"]
    async fn test_load_seed_data() {
        // This test validates seed_data.sql syntax
        // Skipped by default as it requires RUSTCHAT_TEST_DATABASE_URL
        if let Ok(db_url) = std::env::var("RUSTCHAT_TEST_DATABASE_URL") {
            let pool = sqlx::PgPool::connect(&db_url)
                .await
                .expect("Failed to connect to test DB");

            load_seed_data(&pool)
                .await
                .expect("Failed to load seed data");
        }
    }
}
