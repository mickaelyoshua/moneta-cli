use sqlx::{PgPool, postgres::PgPoolOptions};

pub struct Db {
    pub pool: PgPool,
}

impl Db {
    pub async fn new(
        database_url: &str,
        max_connections: u32,
    ) -> Result<Self, crate::error::AppError> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await?;

        // Execute integrated migrations to the binary at compile time
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }
}
