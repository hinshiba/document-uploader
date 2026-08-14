use axum::{Router, extract::DefaultBodyLimit, http::StatusCode, routing};
use crate::{
    endpoint::{
        EndpointError,
        EndpointResult,
        alive::alive,
        docs::post_document,
        docs_id::get_document_id,
        faculties::get_faculties,
        subjects::{get_subjects, post_subject},
        subjects_id::{delete_subject, put_subject},
    },
    usecase::repository::{
        DocumentFileRepository,
        DocumentRepository,
        FacultyRepository,
        SubjectRepository,
    },
};


/// ドキュメントのアップロードで受け付けるリクエストボディ全体の上限
/// 
/// axumの既定は2MB (https://docs.rs/axum/0.8.9/axum/extract/struct.Multipart.html#large-files)
const MAX_UPLOAD_BODY_SIZE: usize = 100 * 1024 * 1024;

pub fn api_router<S>(state: S) -> Router
where S: FacultyRepository + SubjectRepository + DocumentRepository + DocumentFileRepository + Clone + 'static
{
    Router::new()
        .route("/alive", routing::get(alive))
        .route("/faculties", routing::get(get_faculties::<S>))
        .route("/subjects", routing::get(get_subjects::<S>).post(post_subject::<S>))
        .route("/subjects/{subjectId}", routing::put(put_subject::<S>).delete(delete_subject::<S>))
        .route(
            "/docs",
            routing::post(post_document::<S>).layer(DefaultBodyLimit::max(MAX_UPLOAD_BODY_SIZE)),
        )
        .route("/docs/{id}", routing::get(get_document_id::<S>))
        .with_state(state)
        // nestの外側のfallbackへ落として静的ファイル配信されないようにする
        .fallback(api_not_found)
}

#[tracing::instrument(ret(level = "info"))]
async fn api_not_found() -> EndpointResult<()> {
    (
        StatusCode::NOT_FOUND,
        Err(EndpointError {
            message: "not found".to_owned(),
            details: None,
        }),
    )
}