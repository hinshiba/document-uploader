use std::str::FromStr;

use axum::{
    Json,
    Router,
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::domain::{
    Grade,
    Id,
    Num,
    Term,
    Year,
    document::{DocumentFileType, DocumentMetadata, ExamType},
    faculty::Faculty,
    subject::Subject,
};
use crate::infrastructure::sqlite_repository::SqliteRepository;
use crate::usecase::app::get_faculties::GetFacultiesUseCase;
use crate::usecase::app::get_subjects::{GetSubjectsOption, GetSubjectsUseCase};
use crate::usecase::app::store_document::{
    StoreDocumentInput,
    StoreDocumentInputFile,
    StoreDocumentUseCase,
};

#[derive(Clone)]
pub struct AppState {
    repository: SqliteRepository,
}

impl AppState {
    pub fn new(repository: SqliteRepository) -> Self {
        Self { repository }
    }
}

/// `/api/v1`配下のルータを構築する．寛容なCORSを付与する．
pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/alive", get(alive))
        .route("/faculties", get(get_faculties))
        .route("/subjects", get(get_subjects))
        .route("/docs", post(post_docs))
        .with_state(state);

    Router::new().nest("/api/v1", api).layer(cors)
}

// ---- DTO (OpenAPIのフィールド名に厳密一致) ----

#[derive(Debug, Serialize)]
struct MajorDto {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct FacultyDto {
    id: String,
    name: String,
    majors: Vec<MajorDto>,
}

impl From<Faculty> for FacultyDto {
    fn from(f: Faculty) -> Self {
        Self {
            id: f.id().id().to_string(),
            name: f.name().to_owned(),
            majors: f
                .majors()
                .iter()
                .map(|m| MajorDto {
                    id: m.id().id().to_string(),
                    name: m.name().to_owned(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct SubjectDto {
    id: String,
    name: String,
    faculty: String,
    major: String,
    grade: i64,
    term: i64,
}

impl From<Subject> for SubjectDto {
    fn from(s: Subject) -> Self {
        Self {
            id: s.id().id().to_string(),
            name: s.name().to_owned(),
            faculty: s.faculty_id().id().to_string(),
            major: s.major_id().id().to_string(),
            grade: *s.grade().grade(),
            term: *s.term().term(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SubjectsQuery {
    faculty: Uuid,
    #[serde(default)]
    major: Option<Uuid>,
    #[serde(default)]
    grade: Option<i64>,
    #[serde(default)]
    term: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct DocumentMetadataDto {
    faculty: Uuid,
    major: Uuid,
    year: i64,
    term: i64,
    grade: i64,
    subject: Uuid,
    teacher: String,
    examtype: String,
    isanswer: bool,
    num: i64,
}

impl DocumentMetadataDto {
    fn into_domain(self) -> anyhow::Result<DocumentMetadata> {
        Ok(DocumentMetadata::new(
            Id::new(self.faculty),
            Id::new(self.major),
            Year::new(self.year)?,
            Term::new(self.term)?,
            Grade::new(self.grade)?,
            Id::new(self.subject),
            self.teacher,
            ExamType::from_str(&self.examtype)?,
            self.isanswer,
            Num::new(self.num)?,
        ))
    }
}

// ---- ハンドラ ----

async fn alive() -> &'static str {
    "ok"
}

async fn get_faculties(State(state): State<AppState>) -> Result<Json<Vec<FacultyDto>>, AppError> {
    let usecase = GetFacultiesUseCase::new(state.repository.clone());
    let faculties = usecase.execute().await?;
    Ok(Json(faculties.into_iter().map(FacultyDto::from).collect()))
}

async fn get_subjects(
    State(state): State<AppState>,
    Query(query): Query<SubjectsQuery>,
) -> Result<Json<Vec<SubjectDto>>, AppError> {
    let usecase = GetSubjectsUseCase::new(state.repository.clone());
    let option = GetSubjectsOption {
        faculty_id: query.faculty,
        major_id: query.major,
        grade: query.grade,
        term: query.term,
    };
    let subjects = usecase.execute(option).await?;
    Ok(Json(subjects.into_iter().map(SubjectDto::from).collect()))
}

async fn post_docs(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    let mut metadata: Option<DocumentMetadataDto> = None;
    let mut files: Vec<StoreDocumentInputFile> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("invalid multipart body: {e}")))?
    {
        let field_name = field.name().unwrap_or_default().to_owned();

        if field_name == "metadata" {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::bad_request(format!("failed to read metadata: {e}")))?;
            let dto: DocumentMetadataDto = serde_json::from_slice(&bytes)
                .map_err(|e| AppError::bad_request(format!("invalid metadata JSON: {e}")))?;
            metadata = Some(dto);
        } else {
            // "files" / "files[]" などファイルフィールド
            let file_name = field.file_name().unwrap_or_default().to_owned();
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::bad_request(format!("failed to read file: {e}")))?;

            // 拡張子から種別を推定．破棄前提のため不明時はTxt扱いで受理する．
            let ext = file_name
                .rsplit('.')
                .next()
                .unwrap_or_default()
                .to_lowercase();
            let file_type = DocumentFileType::from_str(&ext).unwrap_or(DocumentFileType::Txt);

            files.push(StoreDocumentInputFile {
                file_type,
                content: bytes.to_vec(),
            });
        }
    }

    let metadata = metadata
        .ok_or_else(|| AppError::bad_request("metadata field is required"))?
        .into_domain()
        .map_err(|e| AppError::bad_request(format!("invalid metadata value: {e}")))?;

    if files.is_empty() {
        return Err(AppError::bad_request("at least one file is required"));
    }

    let usecase = StoreDocumentUseCase::new(state.repository.clone());
    usecase.execute(StoreDocumentInput { metadata, files }).await?;

    Ok(StatusCode::CREATED)
}

// ---- エラー ----

#[derive(Debug)]
enum AppError {
    BadRequest(String),
    Internal(anyhow::Error),
}

impl AppError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e)
    }
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::Internal(e) => {
                tracing::error!(error = ?e, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_owned(),
                )
            }
        };
        (status, Json(ErrorBody { message })).into_response()
    }
}
