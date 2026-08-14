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
};

mod domain;
mod endpoint;
mod infrastructure;
mod usecase;

use std::sync::Arc;
use infrastructure::repository::ExampleRepository;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 同名の環境変数が登録されている場合はそちらが優先されることに注意
    let _ = dotenvy::dotenv();

    init_tracing_subscriber();
    let listener = tokio::net::TcpListener::bind(env::var("LISTEN_ADDR").context("LISTEN_ADDR is not set.")?).await?;


    let state = Arc::new(init_example_repository()?);

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

fn init_example_repository() -> anyhow::Result<ExampleRepository> {
    let save_dir = std::path::PathBuf::from("save_dir");

    Ok(ExampleRepository::new(save_dir)?)
}

use usecase::repository::*;

impl<I: DocumentFileRepository> DocumentFileRepository for Arc<I> {
    async fn store_document_file(&self, content: Vec<u8>, file_type: domain::document::DocumentFileType) -> anyhow::Result<domain::document::DocumentFile> {
        <I as DocumentFileRepository>::store_document_file(&self, content, file_type).await
    }
    async fn get_document_file_content(&self, document_file: &domain::document::DocumentFile) -> anyhow::Result<Vec<u8>> {
        <I as DocumentFileRepository>::get_document_file_content(&self, document_file).await
    }
}

impl<I: DocumentRepository> DocumentRepository for Arc<I> {
    async fn store_document(&self, document: domain::document::Document) -> anyhow::Result<()> {
        <I as DocumentRepository>::store_document(&self, document).await
    }
    async fn find_document_by_id(&self, document_id: &domain::Id<domain::document::Document>) -> anyhow::Result<Option<domain::document::Document>> {
        <I as DocumentRepository>::find_document_by_id(&self, document_id).await
    }
}

impl<I: FacultyRepository> FacultyRepository for Arc<I> {
    async fn list_faculties(&self) -> anyhow::Result<Vec<domain::major::Faculty>> {
        <I as FacultyRepository>::list_faculties(&self).await
    }
}

impl<I: SubjectRepository> SubjectRepository for Arc<I> {
    async fn list_subjects(&self) -> anyhow::Result<Vec<domain::subject::Subject>> {
        <I as SubjectRepository>::list_subjects(&self).await
    }
    async fn create_subject(&self, subject: domain::subject::Subject) -> anyhow::Result<()> {
        <I as SubjectRepository>::create_subject(&self, subject).await
    }
    async fn delete_subject(&self, subject_id: domain::Id<domain::subject::Subject>) -> anyhow::Result<domain::subject::Subject> {
        <I as SubjectRepository>::delete_subject(&self, subject_id).await
    }
    async fn search_subjects(&self, option: SearchSubjectOption) -> anyhow::Result<Vec<domain::subject::Subject>> {
        <I as SubjectRepository>::search_subjects(&self, option).await
    }
    async fn update_subject(&self, subject_id: domain::Id<domain::subject::Subject>, content: UpdateSubjectContent) -> anyhow::Result<domain::subject::Subject> {
        <I as SubjectRepository>::update_subject(&self, subject_id, content).await
    }
}
