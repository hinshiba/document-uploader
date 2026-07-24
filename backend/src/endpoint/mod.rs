use serde::Serialize;

pub mod alive;
pub mod dto;
pub mod docs;
pub mod docs_id;
pub mod faculties;
pub mod subjects;

pub type EndpointResult<T> = (axum::http::StatusCode, Result<T, EndpointError>);

#[derive(Debug, Clone, Hash, Serialize)]
pub struct EndpointError {
    pub message: String,
    pub details: Option<String>,
}

impl axum::response::IntoResponse for EndpointError {
    fn into_response(self) -> axum::response::Response {
        axum::response::Json(self).into_response()
    }
}
