use axum::{
    extract,
    http::StatusCode,
    response::{
        self,
        IntoResponse,
    },
};
use serde::Deserialize;

use crate::{
    domain::{
        Id,
        Grade,
        Term,
        subject::Subject,
    },
    usecase::{
        app::{
            update_subject::{
                UpdateSubjectInput,
                UpdateSubjectOutput,
                UpdateSubjectUseCase,
            },
            delete_subject::{
                DeleteSubjectInput,
                DeleteSubjectOutput,
                DeleteSubjectUseCase,
            }
        },
        repository::SubjectRepository,
    }
};
use super::{
    dto::subject::SubjectDto,
    EndpointError,
    EndpointResult,
};

#[derive(Debug, Clone, Hash, Deserialize)]
pub struct PutSubjectsIdInput {
    pub name: String,
    pub course_code: String,
    pub faculty: uuid::Uuid,
    pub major: uuid::Uuid,
    pub grade: i64,
    pub term: i64,
}

impl PutSubjectsIdInput {
    fn to_update_subject_input(&self, subject_id: uuid::Uuid) -> Result<UpdateSubjectInput, EndpointError> {
        let update_subject_input = UpdateSubjectInput {
            id: Id::new(subject_id),
            name: self.name.clone(),
            course_code: self.course_code.clone(),
            faculty_id: Id::new(self.faculty),
            major_id: Id::new(self.major),
            grade: Grade::new(self.grade).map_err(parse_error_to_endpoint_error)?,
            term: Term::new(self.term).map_err(parse_error_to_endpoint_error)?,
        };

        Ok(update_subject_input)
    }
}

#[tracing::instrument(skip(repo), ret(level="info"))]
pub async fn put_subject<I: SubjectRepository>(
    extract::State(repo): extract::State<I>,
    extract::Path(subject_id): extract::Path<uuid::Uuid>,
    extract::Json(content): extract::Json<PutSubjectsIdInput>,
) -> EndpointResult<impl IntoResponse> {
    let update_subject_input = match content.to_update_subject_input(subject_id) {
        Ok(ret) => ret,
        Err(err) => return error_with_400(err)
    };

    match UpdateSubjectUseCase::new(repo)
        .execute(update_subject_input)
        .await
        .map(convert_update_subject_output)
    {
        Ok(Ok(subject)) => {
            ( StatusCode::OK
            , Ok(response::Json(SubjectDto {
                id: subject.id().id().clone(),
                name: subject.name().to_owned(),
                faculty: subject.faculty_id().id().clone(),
                major: subject.major_id().id().clone(),
                grade: subject.grade().grade().clone(),
                term: subject.term().term().clone()
              }))
            )
        }
        Ok(Err(err)) => error_with_400(err),
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

#[tracing::instrument(skip(repo), ret(level="info"))]
pub async fn delete_subject<I: SubjectRepository>(
    extract::State(repo): extract::State<I>,
    extract::Path(subject_id): extract::Path<uuid::Uuid>,
) -> EndpointResult<impl IntoResponse> {
    match DeleteSubjectUseCase::new(repo).execute(
            DeleteSubjectInput {
                subject_id: Id::new(subject_id)
            }
        ).await
    {
        Ok(DeleteSubjectOutput::Deleted) => (StatusCode::NO_CONTENT, Ok(())),
        Ok(DeleteSubjectOutput::ErrReferencedByDocuments) => (
            StatusCode::CONFLICT,
            Err(EndpointError {
                message: "subject is still referenced by documents".to_owned(),
                details: Some("reassign or delete them before deleting this subject".to_owned())
            })
        ),
        Ok(DeleteSubjectOutput::ErrSubjectNotExist(_subject_id)) => (
            StatusCode::NOT_FOUND,
            Err(EndpointError {
                message: "invalid subject_id".to_owned(),
                details: Some("subject does not exist".to_owned()),
            })
        ),
        Err(err) => {
            tracing::error!("{}", err);

            ( StatusCode::INTERNAL_SERVER_ERROR
            , Err(EndpointError {
                message: "unexpected error occured".to_owned(),
                details: None,
              })
            )
        }
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
fn convert_update_subject_output(o: UpdateSubjectOutput) -> Result<Subject, EndpointError> {
    use UpdateSubjectOutput::*;

    match o {
        Updated(subject) => Ok(subject),
        ErrCourseCodeNonUnique(course_code) => Err(
            EndpointError {
                message: "duplicate check failed".to_owned(),
                details: Some(format!("subject which have course_code '{}' already exists", course_code)),
            }
        ),
        ErrSubjectNotExist(_subject_id) => Err(
            EndpointError {
                message: "invalid subject_id".to_owned(),
                details: Some("subject does not exist".to_owned()),
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
