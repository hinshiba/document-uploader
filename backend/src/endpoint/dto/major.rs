use serde::{
    Deserialize,
    Serialize,
};

use crate::domain::major::Major;

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct MajorDto {
    #[serde(with="uuid::serde::hyphenated")]
    pub id: uuid::Uuid,
    pub name: String,
}

impl MajorDto {
    pub fn from_domain(major: &Major) -> Self {
        Self {
            id: major.id().id().clone(),
            name: major.name().to_owned(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_faculty_dto() {
        let major_dto = serde_json::from_str::<MajorDto>(
            r#"
            {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "name": "情電数理系/情報工学コース"
            }
            "#
        );

        dbg!(&major_dto);
        assert!(major_dto.is_ok());
    }
}
