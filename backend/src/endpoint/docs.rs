use axum::extract::{
    Multipart,
    State,
};
use serde::Deserialize;

use crate::usecase::{
    app::store_document::{
        StoreDocumentInput,
        StoreDocumentInputFile,
        StoreDocumentUseCase,
    },
    repository::{
        DocumentFileRepository,
        DocumentRepository,
    },
};
use super::{
    EndpointError,
    EndpointResult,
};

#[tracing::instrument(skip_all, ret(level="info"))]
pub async fn put_documents<I: DocumentFileRepository + DocumentRepository>(
    State(repo): State<I>,
    mut multipart: Multipart,
) -> EndpointResult<()> {
    todo!()
}
