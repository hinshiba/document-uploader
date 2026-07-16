use std::cell::OnceCell;

use axum::{
    extract::{
        Multipart,
        State,
        multipart::Field,
    },
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    domain::{
        Id,
        Year,
        Term,
        Grade,
        Num,
        document::{
            DocumentFileType,
            DocumentMetadata,
        },
    },
    usecase::{
        app::store_document::{
            StoreDocumentInput,
            StoreDocumentInputFile,
            StoreDocumentUseCase,
        },
        repository::{
            DocumentFileRepository,
            DocumentRepository,
        },
    }
};
use super::{
    EndpointError,
    EndpointResult,
};

#[derive(Debug, Clone, Hash, Deserialize)]
pub struct DocumentMetadataInput {
    faculty: uuid::Uuid,
    major: uuid::Uuid,
    year: i64,
    term: i64,
    grade: i64,
    subject: uuid::Uuid,
    teacher: String,
    examtype: String,
    isanswer: bool,
    num: i64,
}

impl DocumentMetadataInput {
    pub fn to_document_metadata(&self) -> Result<DocumentMetadata, EndpointError> {
        let year = Year::new(self.year).map_err(Self::parse_error_to_endpoint_error)?;
        let term = Term::new(self.term).map_err(Self::parse_error_to_endpoint_error)?;
        let grade = Grade::new(self.grade).map_err(Self::parse_error_to_endpoint_error)?;
        let examtype = self.examtype.parse().map_err(Self::parse_error_to_endpoint_error)?;
        let num = Num::new(self.num).map_err(Self::parse_error_to_endpoint_error)?;

        Ok(DocumentMetadata::new(
            Id::new(self.faculty),
            Id::new(self.major),
            year,
            term,
            grade,
            Id::new(self.subject),
            self.teacher.clone(),
            examtype,
            self.isanswer,
            num,
        ))
    }

    #[inline]
    fn parse_error_to_endpoint_error(e: impl std::error::Error) -> EndpointError {
        EndpointError {
            message: "multi part error".to_owned(),
            details: Some(e.to_string()),
        }
    }
}

#[tracing::instrument(skip_all, ret(level="info"))]
pub async fn post_document<I: DocumentFileRepository + DocumentRepository>(
    State(repo): State<I>,
    mut multipart: Multipart,
) -> EndpointResult<impl IntoResponse> {
    let mut files: Vec<(DocumentFileType, Vec<u8>)> = Vec::new();
    let metadata: OnceCell<DocumentMetadataInput> = OnceCell::new();

    loop {
        // multipart/form-dataのフィールドを取得
        let field = match multipart.next_field().await {
            Ok(field) => field,
            Err(err) => {
                return error_with_400(
                    EndpointError {
                        message: "multi part error".to_owned(),
                        details: Some(err.to_string()),
                    }
                )
            }
        };

        // 終端へ到達したらループを終了
        let Some(field) = field
        else {
            break;
        };

        // フィールド名を取得し、それによって分岐
        let Some(field_name) = field.name()
        else {
            return error_with_400(
                EndpointError {
                    message: "multi part error".to_owned(),
                    details: Some("no field name in form data".to_owned()),
                }
            )
        };

        match field_name {
            "metadata" => {
                let metadata_input = match get_metadata(field).await {
                    Ok(metadata) => metadata,
                    Err(err) => {
                        return error_with_400(err);
                    }
                };

                // metadataが既に現れていたらエラーを投げる
                if metadata.set(metadata_input).is_err() {
                    return error_with_400(
                        EndpointError {
                            message: "multi part error".to_owned(),
                            details: Some("metadata can be given exactly once".to_owned()),
                        }
                    );
                }
            },
            "files" => {
                match get_file(field).await {
                    Ok(file) => files.push(file),
                    Err(err) => {
                        return error_with_400(err);
                    }
                }
            },
            unexpected => {
                return error_with_400(
                    EndpointError {
                        message: "multi part error".to_owned(),
                        details: Some(format!("found unexpected field name '{}'", unexpected)),
                    }
                );
            }
        }
    }

    let files = files;

    // metadataが見つからなかったらエラーを投げる
    let Some(metadata) = metadata.into_inner()
    else {
        return error_with_400(
            EndpointError {
                message: "missing required field".to_owned(),
                details: Some("'metadata' is required".to_owned()),
            }
        )
    };

    // `metadata`をユースケースの引数に合うようにパース
    let metadata = match metadata.to_document_metadata() {
        Ok(metadata) => metadata,
        Err(err) => {
            return error_with_400(err)
        }
    };

    match StoreDocumentUseCase::new(repo).execute(
        StoreDocumentInput {
            metadata,
            files: files.into_iter()
                    .map(|(ty, c)| {
                        StoreDocumentInputFile {
                            file_type: ty,
                            content: c,
                        }
                    }).collect(),
        },
    ).await {
        Ok(()) => {
            return (StatusCode::CREATED, Ok(()))
        },
        Err(err) => {
            tracing::error!("{}", err);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Err(EndpointError {
                    message: "unexpected error occured".to_owned(),
                    details: None,
                })
            )
        }
    }
}

// 以下helper functions

#[inline]
fn error_with_400<T>(e: EndpointError) -> EndpointResult<T> {
    (StatusCode::BAD_REQUEST, Err(e))
}

async fn get_metadata<'a>(field: Field<'a>) -> Result<DocumentMetadataInput, EndpointError> {
    // text形式でmetadataをmultipart/form-dataから取得
    let metadata = match field.text().await {
        Ok(metadata) => metadata,
        Err(err) => {
            return Err(EndpointError {
                message: "multi part error".to_owned(),
                details: Some(err.to_string()),
            });
        }
    };

    // text形式の`metadata`を`DocumentMetadataInput`へdeserialise
    let metadata = match serde_json::from_str(&metadata) {
        Ok(metadata) => metadata,
        Err(err) => {
            return Err(EndpointError {
                message: "multi part error".to_owned(),
                details: Some(err.to_string()),
            })
        }
    };
    
    Ok(metadata)
}

async fn get_file<'a>(field: Field<'a>) -> Result<(DocumentFileType, Vec<u8>), EndpointError> {
    // ファイル名を取得
    let Some(file_name) = field.file_name()
    else {
        return Err(EndpointError {
            message: "multi part error".to_owned(),
            details: Some("'file_name' is required".to_owned()),
        });
    };

    // ファイル名に拡張子が含まれていると仮定し、そこからファイルタイプを取得
    let Some(file_type) = file_name.rsplit_terminator('.').next()
    else {
        return Err(EndpointError {
            message: "multi part error".to_owned(),
            details: Some("cannot detect file type".to_owned()),
        });
    };

    // `String`から`DocumentFileType`へパース
    let file_type = match file_type.parse::<DocumentFileType>() {
        Ok(file_type) => file_type,
        Err(err) => {
            return Err(EndpointError {
                message: "multi part error".to_owned(),
                details: Some(err.to_string()),
            });
        }
    };

    let content = match field.bytes().await {
        Ok(content) => content.to_vec(),
        Err(err) => {
            return Err(EndpointError {
                message: "multi part error".to_owned(),
                details: Some(err.to_string()),
            })
        }
    };
    
    Ok((file_type, content))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_document_metadata_input() {
        let document_metadata_input = serde_json::from_str::<DocumentMetadataInput>(
            r#"
            {
                "faculty": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
                "major": "550e8400-e29b-41d4-a716-446655440000",
                "year": 2025,
                "term": 2,
                "grade": 2,
                "subject": "9b2e4c6a-1f3d-4e5b-8a7c-0d1e2f3a4b5c",
                "teacher": "岡山 聖彦",
                "examtype": "final",
                "isanswer": false,
                "num": 1
            }
            "#
        );

        dbg!(&document_metadata_input);
        assert!(document_metadata_input.is_ok());

        let document_metadata_input = document_metadata_input.unwrap();

        let document_metadata = document_metadata_input.to_document_metadata();
        
        dbg!(&document_metadata);
        assert!(document_metadata.is_ok());
    }

    #[test]
    fn parse_file_type() {
        use DocumentFileType::*;

        let testcase = vec![
            ("jpg", Jpeg),
            ("doc", Doc),
            ("png", Png),
            ("md", Markdown),
        ];

        for ty in testcase {
            let file_name = format!("files.{}", ty.0);
            let file_type = file_name.rsplit_terminator('.').next();

            assert!(file_type.is_some());
            let file_type = file_type.unwrap();

            let file_type = file_type.parse::<DocumentFileType>();
            
            dbg!(&file_type);
            assert!(file_type.is_ok());
            let file_type = file_type.unwrap();

            assert_eq!(file_type, ty.1);
        }
    }
}
