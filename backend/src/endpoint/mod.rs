use serde::Serialize;

pub mod alive;
pub mod dto;

#[derive(Debug, Clone, Hash, Serialize)]
pub struct EndpointError {
    pub message: String,
    pub details: Option<String>,
}

pub type EndpointResult<T> = Result<T, EndpointError>;
