use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};

pub mod entities;
pub mod repos;

pub async fn init_pool() -> anyhow::Result<DatabaseConnection> {
    let base_url = std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?;

    let sep = if base_url.contains('?') { '&' } else { '?' };
    let url = format!("{base_url}{sep}application_name=tokimo-app-helloworld");

    let mut opts = ConnectOptions::new(url);
    opts.max_connections(4).min_connections(1).sqlx_logging(false);

    Ok(Database::connect(opts).await?)
}

pub async fn init_schema(db: &DatabaseConnection) -> anyhow::Result<()> {
    let schema = std::env::var("DB_SCHEMA").unwrap_or_else(|_| "helloworld".to_string());

    if !schema.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        anyhow::bail!("DB_SCHEMA contains invalid characters: {schema:?}");
    }

    let ddl = [
        format!(r#"CREATE SCHEMA IF NOT EXISTS "{schema}""#),
        format!(
            r#"CREATE TABLE IF NOT EXISTS "{schema}".items (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                content TEXT NOT NULL,
                user_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )"#
        ),
        format!(
            r#"ALTER TABLE "{schema}".items ADD COLUMN IF NOT EXISTS user_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'"#
        ),
        format!(r#"CREATE INDEX IF NOT EXISTS items_created_at_idx ON "{schema}".items (created_at DESC)"#),
        format!(r#"CREATE INDEX IF NOT EXISTS items_user_id_idx ON "{schema}".items (user_id)"#),
    ];

    for sql in ddl {
        db.execute_raw(Statement::from_string(DatabaseBackend::Postgres, sql))
            .await?;
    }

    Ok(())
}
