mod domain;
mod infrastructure;
mod usecase;
mod web;

use sqlx::sqlite::SqlitePoolOptions;

use crate::infrastructure::sqlite_repository::SqliteRepository;
use crate::web::AppState;

const DEFAULT_DATABASE_URL: &str = "sqlite://data.db?mode=rwc";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // backend/.env があれば読み込む(存在しなくてもよい)
    let _ = dotenvy::dotenv();

    init_tracing_subscriber();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_owned());
    tracing::info!("using database: {}", database_url);

    // `mode=rwc` によりDBファイルが無ければ作成される
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // 起動時にmigrationを適用する
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("migrations applied");

    let state = AppState::new(SqliteRepository::new(pool));
    let app = web::router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("start listening on http://{}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing_subscriber() {
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_env_var("RUST_LOG")
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
        .init();
}
