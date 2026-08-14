use serde::{
    Deserialize,
    Serialize,
};

use crate::domain::document::{
    ExamType,
    Document,
    DocumentMetadata,
};

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct ExamTypeDto(pub String);

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct DocumentMetadataDto {
    #[serde(with="uuid::serde::hyphenated")]
    pub faculty: uuid::Uuid,
    #[serde(with="uuid::serde::hyphenated")]
    pub major: uuid::Uuid,
    pub year: i64,
    pub term: i64,
    pub grade: i64,
    #[serde(with="uuid::serde::hyphenated")]
    pub subject: uuid::Uuid,
    pub teacher: String,
    pub examtype: ExamTypeDto,
    pub isanswer: bool,
    pub num: i64,
}

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct DocumentDto {
    #[serde(with="uuid::serde::hyphenated")]
    pub id: uuid::Uuid,
    pub metadata: DocumentMetadataDto,
}

impl ExamTypeDto {
    pub fn from_domain(domain: &ExamType) -> Self {
        match *domain {
            ExamType::Quiz => Self("quiz".to_owned()),
            ExamType::MidTerm => Self("midterm".to_owned()),
            ExamType::FinalTerm => Self("final".to_owned()),
            ExamType::Other => Self("other".to_owned()),
        }
    }
}

impl DocumentMetadataDto {
    pub fn from_domain(domain: &DocumentMetadata) -> Self {
        Self {
            faculty: domain.faculty_id().id().clone(),
            major: domain.major_id().id().clone(),
            year: domain.year().year().clone(),
            term: domain.term().term().clone(),
            grade: domain.grade().grade().clone(),
            subject: domain.subject_id().id().clone(),
            teacher: domain.teacher().to_owned(),
            examtype: ExamTypeDto::from_domain(domain.exam_type()),
            isanswer: domain.is_answer().clone(),
            num: domain.num().num().clone(),
        }
    }
}

impl DocumentDto {
    pub fn from_domain(domain: &Document) -> Self {
        Self {
            id: domain.id().id().clone(),
            metadata: DocumentMetadataDto::from_domain(domain.metadata()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_document_dto() {
        let doc = serde_json::json!(
            {
                "id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
                "metadata": {
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
            }
        );

        assert!( dbg!(serde_json::from_value::<DocumentDto>(doc.clone())).is_ok() );
        assert!( dbg!(serde_json::to_string(&doc)).is_ok() );
        assert_eq!(
            serde_json::to_value(
                serde_json::from_value::<DocumentDto>(doc.clone()).unwrap()
            ).unwrap(),
            doc,
        );
    }
}
