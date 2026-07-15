use serde::{
    Deserialize,
    Serialize,
};

use crate::domain::subject::Subject;

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct SubjectDto {
    #[serde(with="uuid::serde::hyphenated")]
    pub id: uuid::Uuid,
    pub name: String,
    #[serde(with="uuid::serde::hyphenated")]
    pub faculty: uuid::Uuid,
    #[serde(with="uuid::serde::hyphenated")]
    pub major: uuid::Uuid,
    pub grade: i64,
    pub term: i64,
}

impl SubjectDto {
    pub fn from_domain(subject: &Subject) -> Self {
        Self {
            id: subject.id().id().clone(),
            name: subject.name().to_owned(),
            faculty: subject.faculty_id().id().clone(),
            major: subject.major_id().id().clone(),
            grade: *subject.grade().grade(),
            term: *subject.term().term(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_subject_dto() {
        let subject_dto = serde_json::from_str::<SubjectDto>(
            r#"
            {
                "id": "9b2e4c6a-1f3d-4e5b-8a7c-0d1e2f3a4b5c",
                "name": "線形代数",
                "faculty": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
                "major": "550e8400-e29b-41d4-a716-446655440000",
                "grade": 1,
                "term": 1
            }
            "#
        );

        dbg!(&subject_dto);
        assert!(subject_dto.is_ok());
    }
}
