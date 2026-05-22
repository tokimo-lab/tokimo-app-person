use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub mod entities;
pub mod repos;

/// 连接 host 提供的 PostgreSQL 数据库。
///
/// Host 启动 app 进程时注入 `DATABASE_URL` + `TOKIMO_APP_SCHEMA`，
/// 并已经在主进程侧完成所有 schema migration。这里只负责连库、
/// 把每条连接的 `search_path` 钉到本 app 自己的 schema。
///
/// CLI 直连模式（没有 host）下 `TOKIMO_APP_SCHEMA` 可能缺失，
/// 此时 fallback 到固定 schema 名 "helloworld"。
pub async fn init_pool() -> anyhow::Result<DatabaseConnection> {
    let base_url = std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?;
    let schema = std::env::var("TOKIMO_APP_SCHEMA").unwrap_or_else(|_| "helloworld".to_string());

    // schema 已被 host 用 `[a-z_][a-z0-9_]*` 校验过；CLI fallback 是硬编码字面量。
    // 因此可以放心拼接进 libpq 风格的 `options=-c search_path=...` 参数里。
    // 编码：空格 -> %20，等号 -> %3D，双引号 -> %22，逗号 -> %2C。
    let sep = if base_url.contains('?') { '&' } else { '?' };
    let url = format!(
        "{base_url}{sep}application_name=tokimo-app-helloworld\
         &options=-c%20search_path%3D%22{schema}%22%2Cpublic"
    );

    let mut opts = ConnectOptions::new(url);
    opts.max_connections(4).min_connections(1).sqlx_logging(false);

    Ok(Database::connect(opts).await?)
}
