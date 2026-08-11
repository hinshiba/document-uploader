use std::env;

use anyhow::Context;

mod domain;
mod endpoint;
mod infrastructure;
mod usecase;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    init_tracing_subscriber();
    let listener = tokio::net::TcpListener::bind(env::var("LISTEN_ADDR").context("LISTEN_ADDR is not set.")?).await?;
    let pgpool = sqlx::PgPool::connect(&env::var("DATABASE_URL").context("DATABASE_URL is not set.")?).await?;

    let app = axum::Router::new()
        .route("/", axum::routing::get(|| async { "Hello, World!" }));

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
