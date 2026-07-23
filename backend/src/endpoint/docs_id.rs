use axum::{
    extract::{
        State,
        Path,
    },
    http::{
        header,
        StatusCode,
    },
    response::IntoResponse,
};

use crate::{
    domain::Id,
    usecase::{
        app::get_zip_document::{
            GetZipDocumentInput,
            GetZipDocumentUseCase,
        },
        repository::{
            DocumentRepository,
            DocumentFileRepository,
        },
    },
};
use super::{
    EndpointError,
    EndpointResult,
};

#[tracing::instrument(skip_all, fields(document_id))]
pub async fn get_document_id<I: DocumentRepository + DocumentFileRepository>(
    State(repo): State<I>,
    Path(document_id): Path<uuid::Uuid>,
) -> EndpointResult<impl IntoResponse> {
    match GetZipDocumentUseCase::new(repo)
        .execute(GetZipDocumentInput {
            document_id: Id::new(document_id),
        }).await
    {
        Ok(Some(output)) => {
            let encoded_filename = percent_encoding::utf8_percent_encode(
                    &output.name,
                    percent_encoding::NON_ALPHANUMERIC,
                ).to_string();

            ( StatusCode::OK
            , Ok((
                [ (header::CONTENT_DISPOSITION, format!("attachment; filename*=UTF-8''{}", encoded_filename))
                , (header::CONTENT_TYPE, "application/zip".to_owned())
                ],
                output.content
            ))
            )
        },
        Ok(None) => {
            tracing::info!("Not Found");

            ( StatusCode::NOT_FOUND
            , Err(EndpointError {
                message: "document does not exist".to_owned(),
                details: Some(format!("id: {}", document_id)),
            })
            )
        },
        Err(err) => {
            tracing::error!("{}", err);

            ( StatusCode::INTERNAL_SERVER_ERROR
            , Err(EndpointError {
                message: "unexpected error occured".to_owned(),
                details: None,
            })
            )
        },
    }
}
