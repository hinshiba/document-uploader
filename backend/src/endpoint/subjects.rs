use axum::extract::{
    Json,
    State,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::usecase::{
    app::get_subjects::{
        GetSubjectsOption,
        GetSubjectsUseCase,
    },
    repository::SubjectRepository,
};
use super::{
    dto::subject::SubjectDto,
    EndpointError,
    EndpointResult,
};

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct Input {
    pub faculty: uuid::Uuid,
    pub major: Option<uuid::Uuid>,
    pub grade: Option<i64>,
    pub term: Option<i64>,
}

impl Input {
    pub fn to_get_subjects_option(&self) -> GetSubjectsOption {
        GetSubjectsOption {
            faculty_id: self.faculty,
            major_id: self.major,
            grade: self.grade,
            term: self.term,
        }
    }
}

#[tracing::instrument(skip(repo), ret(level="info"))]
pub async fn get_subjects<I: SubjectRepository>(
    State(repo): State<I>,
    Json(input): Json<Input>,
) -> EndpointResult<Vec<SubjectDto>> {
    let option = input.to_get_subjects_option();

    let subjects = match GetSubjectsUseCase::new(repo).execute(option).await {
        Ok(subjects) => subjects,
        Err(err) => {
            tracing::error!("{}", err);

            return Err(EndpointError {
                message: "unexpected error occured".to_owned(),
                details: None,
            });
        }
    };

    return Ok(
        subjects.into_iter()
            .map(|s| SubjectDto::from_domain(&s))
            .collect()
    )
}
