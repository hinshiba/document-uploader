mod domain;
mod endpoint;
mod infrastructure;
mod usecase;

use std::sync::Arc;
use infrastructure::repository::ExampleRepository;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing_subscriber();

    let repo = Arc::new(ExampleRepository::new("./test".into())?);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    let app = axum::Router::new()
        .route("/", axum::routing::get(|| async { "Hello, World!" }))
        .route("/alive", axum::routing::get(endpoint::alive::alive))
        .route("/faculties", axum::routing::get(endpoint::faculties::get_faculties::<Arc<_>>))
        .route("/subjects", axum::routing::get(endpoint::subjects::get_subjects::<Arc<_>>))
        .route("/docs", axum::routing::post(endpoint::docs::post_document::<Arc<_>>))
        .with_state(repo);

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

use usecase::repository::*;

impl<I: DocumentFileRepository> DocumentFileRepository for Arc<I> {
    async fn store_document_file(&self, content: Vec<u8>, file_type: domain::document::DocumentFileType) -> anyhow::Result<domain::document::DocumentFile> {
        <I as DocumentFileRepository>::store_document_file(&self, content, file_type).await
    }
}

impl<I: DocumentRepository> DocumentRepository for Arc<I> {
    async fn store_document(&self, document: domain::document::Document) -> anyhow::Result<()> {
        <I as DocumentRepository>::store_document(&self, document).await
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
}
