use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{info, instrument};

pub struct Db {
    pub pool: PgPool,
}

impl Db {
    /// Init DB and migrations.
    /// #[instrument] inherits context; skip sensitive args.
    #[instrument(skip(database_url))]
    pub async fn new(
        database_url: &str,
        max_connections: u32,
    ) -> Result<Self, crate::error::AppError> {
        info!("Connecting DB...");
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .map_err(|e| {
                tracing::error!("DB connect failed: {}", e);
                e
            })?;

        info!("Running migrations...");
        // Migrations embedded in binary.
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| {
                tracing::error!("Migration failed: {}", e);
                e
            })?;
        info!("DB ready.");

        Ok(Self { pool })
    }
}
