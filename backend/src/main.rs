//! document-uploaderのエントリポイント
//! 
//! # 必要な環境変数
//! 
//! - LISTEN_ADDR
//! - DATABASE_URL
//! 
//! # Notes
//! 
//! `./public/` にあるファイルが配信される
//!
//! `./save_dir`にすべてのアップロードされたファイルは格納される

use std::env;

use anyhow::Context;
use tower_http::services::ServeDir;

use crate::{
    endpoint::api::api_router, 
    infrastructure::{
        local_fs::LocalFileSystemRepository,
        pgfs::PgFsRepository,
        postgresql::PostgresRepository
    }
};

mod domain;
mod endpoint;
mod infrastructure;
mod usecase;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 同名の環境変数が登録されている場合はそちらが優先されることに注意
    let _ = dotenvy::dotenv();

    init_tracing_subscriber();
    let listener = tokio::net::TcpListener::bind(env::var("LISTEN_ADDR").context("LISTEN_ADDR is not set.")?).await?;
    let pgpool = sqlx::PgPool::connect(&env::var("DATABASE_URL").context("DATABASE_URL is not set.")?).await?;


    let state = PgFsRepository::new(
        PostgresRepository::new(pgpool), 
        LocalFileSystemRepository::new("save_dir".into())?
    );

    let app = axum::Router::new()
        .nest("/api/v1", api_router(state))
        .fallback_service(ServeDir::new("public"));

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
