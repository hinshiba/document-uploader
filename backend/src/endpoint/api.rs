use axum::{Router, routing};
use crate::{
    endpoint::{
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


pub fn api_router<S>(state: S) -> Router
where S: FacultyRepository + SubjectRepository + DocumentRepository + DocumentFileRepository + Clone + 'static
{
    Router::new()
        .route("/alive", routing::get(alive))
        .route("/faculties", routing::get(get_faculties::<S>))
        .route("/subjects", routing::get(get_subjects::<S>).post(post_subject::<S>))
        .route("/subjects/{subjectId}", routing::put(put_subject::<S>).delete(delete_subject::<S>))
        .route("/docs", routing::post(post_document::<S>))
        .route("/docs/{id}", routing::get(get_document_id::<S>))
        .with_state(state)
}