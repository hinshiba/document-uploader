use serde::{
    Deserialize,
    Serialize,
};

use crate::domain::faculty::Faculty;
use super::major::MajorDto;

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct FacultyDto {
    #[serde(with="uuid::serde::hyphenated")]
    pub id: uuid::Uuid,
    pub name: String,
    pub majors: Vec<MajorDto>,
}

impl FacultyDto {
    pub fn from_domain(domain: &Faculty) -> Self {
        Self {
            id: domain.id().id().clone(),
            name: domain.name().to_owned(),
            majors: domain.majors().iter().map(MajorDto::from_domain).collect(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_faculty_dto() {
        let faculty_dto = serde_json::from_str::<FacultyDto>(
            r#"
            {
                "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
                "name": "工学部",
                "majors": [
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440000",
                        "name": "情電数理系/情報工学コース"
                    },
                    {
                        "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
                        "name": "情電数理系/ネットワーク工学コース"
                    }
                ]
            }
            "#
        );

        dbg!(&faculty_dto);
        assert!(faculty_dto.is_ok());
    }
}
