use axum::{
    extract::State,
    http::StatusCode,
    response::{
        IntoResponse,
        Json,
    },
};

use crate::usecase::{
    app::get_faculties::GetFacultiesUseCase,
    repository::FacultyRepository,
};
use super::{
    dto::faculty::FacultyDto,
    EndpointError,
    EndpointResult,
};

#[tracing::instrument(skip_all, ret(level = "info"))]
pub async fn get_faculties<I: FacultyRepository>(
    State(repo): State<I>,
) -> EndpointResult<impl IntoResponse> {
    let faculties = match GetFacultiesUseCase::new(repo).execute().await {
        Ok(faculties) => faculties,
        Err(err) => {
            tracing::error!("{}", err);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(EndpointError {
                    message: "unexpected error occured".to_owned(),
                    details: None,
                })
            );
        }
    };

    return (
        StatusCode::OK,
        Ok(Json(
            faculties.into_iter()
                .map(|f| FacultyDto::from_domain(&f))
                .collect::<Vec<_>>()
        ))
    );
}
