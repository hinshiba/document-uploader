use axum::{
    extract::{
        self,
        Query,
        State,
    },
    http::{
        header,
        StatusCode,
    },
    response::{
        IntoResponse,
        Json,
    },
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    domain::{
        Id,
        Grade,
        Term,
        subject::Subject,
    },
    usecase::{
        app::{
            create_subject::{
                CreateSubjectInput,
                CreateSubjectOutput,
                CreateSubjectUseCase,
            },
            get_subjects::{
                GetSubjectsOption,
                GetSubjectsUseCase,
            },
        },
        repository::SubjectRepository,
    },
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

#[derive(Debug, Clone, Hash, Deserialize)]
pub struct PostInput {
    name: String,
    course_code: String,
    faculty: uuid::Uuid,
    major: uuid::Uuid,
    grade: i64,
    term: i64,
}

impl PostInput {
    fn to_create_subject_input(&self) -> Result<CreateSubjectInput, EndpointError> {
        let create_subject_input = CreateSubjectInput {
            name: self.name.clone(),
            course_code: self.course_code.clone(),
            faculty_id: Id::new(self.faculty),
            major_id: Id::new(self.major),
            grade: Grade::new(self.grade).map_err(parse_error_to_endpoint_error)?,
            term: Term::new(self.term).map_err(parse_error_to_endpoint_error)?,
        };

        Ok(create_subject_input)
    }
}

#[tracing::instrument(skip(repo), ret(level="info"))]
pub async fn get_subjects<I: SubjectRepository>(
    State(repo): State<I>,
    Query(input): Query<Input>,
) -> EndpointResult<impl IntoResponse> {
    let option = input.to_get_subjects_option();

    let subjects = match GetSubjectsUseCase::new(repo).execute(option).await {
        Ok(subjects) => subjects,
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

    (
        StatusCode::OK,
        Ok(Json(
            subjects.into_iter()
                .map(|s| SubjectDto::from_domain(&s))
                .collect::<Vec<_>>()
        ))
    )
}

#[tracing::instrument(skip(repo), ret(level="info"))]
pub async fn post_subject<I: SubjectRepository>(
    State(repo): State<I>,
    extract::Json(input): extract::Json<PostInput>,
) -> EndpointResult<impl IntoResponse> {
    let create_subject_input = match input.to_create_subject_input() {
        Ok(ret) => ret,
        Err(err) => return error_with_400(err),
    };

    match CreateSubjectUseCase::new(repo)
            .execute(create_subject_input)
            .await
            .map(convert_create_subject_output)
    {
        Ok(Ok(subject)) => {
            ( StatusCode::CREATED
            , Ok(([(header::LOCATION, format!("/subjects/{}", subject.id().id().as_hyphenated()))
                  ],
                  Json(SubjectDto {
                    id: subject.id().id().clone(),
                    name: subject.name().to_owned(),
                    faculty: subject.faculty_id().id().clone(),
                    major: subject.major_id().id().clone(),
                    grade: subject.grade().grade().clone(),
                    term: subject.term().term().clone()
                  })
            )))
        },
        Ok(Err(err)) => error_with_400(err),
        Err(err) => {
            tracing::error!("{}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(EndpointError {
                    message: "unexpected error occured".to_owned(),
                    details: None,
                })
            )
        },
    }
}

// 以下 helper functions

#[inline]
fn parse_error_to_endpoint_error(e: impl std::error::Error) -> EndpointError {
    EndpointError {
        message: "validation error".to_owned(),
        details: Some(e.to_string())
    }
}

#[inline]
fn error_with_400<T>(e: EndpointError) -> EndpointResult<T> {
    (StatusCode::BAD_REQUEST, Err(e))
}

#[inline]
fn convert_create_subject_output(o: CreateSubjectOutput) -> Result<Subject, EndpointError> {
    use CreateSubjectOutput::*;

    match o {
        Created(subject) => Ok(subject),
        ErrCourseCodeNonUnique(course_code) => Err(
            EndpointError {
                message: "duplicate check failed".to_owned(),
                details: Some(format!("subject which have course_code '{}' already exists", course_code)),
            }
        ),
        ErrFacultyNotExist(_faculty_id) => Err(
            EndpointError {
                message: "invalid faculty_id".to_owned(),
                details: Some("faculty does not exist".to_owned()),
            }
        ),
        ErrMajorNotExist(_major_id) => Err(
            EndpointError {
                message: "invalid major_id".to_owned(),
                details: Some("major does not exist".to_owned()),
            }
        ),
        ErrFacultyMajorRelation(_faculty_id, _major_id) => Err(
            EndpointError {
                message: "invalid major_id".to_owned(),
                details: Some("major does not belong to faculty".to_owned()),
            }
        ),
    }
}
