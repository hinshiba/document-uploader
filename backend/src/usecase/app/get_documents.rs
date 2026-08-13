use crate::{
    domain::{
        Id,
        Year,
        document::{
            Document,
            ExamType,
        },
    },
    usecase::repository::{
        SearchDocumentOption,
        DocumentRepository,
    },
};

#[derive(Debug)]
pub struct GetDocumentsUseCase<I> {
    repository: I
}

#[derive(Debug, Clone, Hash)]
pub struct GetDocumentsOption {
    pub subject_id: uuid::Uuid,
    pub year: Option<i64>,
    pub teacher: Option<String>,
    pub exam_type: Option<ExamType>,
    pub is_answer: Option<bool>,
}

impl<I> GetDocumentsUseCase<I> {
    pub fn new(repository: I) -> Self {
        Self { repository }
    }
}

impl<I: DocumentRepository> GetDocumentsUseCase<I> {
    #[tracing::instrument(skip(self), ret(level="debug"), err)]
    pub async fn execute(&self, option: GetDocumentsOption) -> anyhow::Result<Vec<Document>> {
        // `option.year`の検証に失敗したら空配列を返す
        let Ok(option_year) = option.year.map(|y| Year::new(y)).transpose()
        else {
            return Ok(vec![]);
        };

        let repo_option = SearchDocumentOption {
            subject_id: Id::new(option.subject_id),
            year: option_year,
            teacher: option.teacher,
            exam_type: option.exam_type,
            is_answer: option.is_answer,
        };

        let result_vec = self.repository.search_documents(repo_option).await?;

        Ok(result_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::document::DocumentMetadata;

    #[test]
    fn transpose_option_result() {
        let a = Some(2026_i64);
        let b = a.map(|y| Year::<DocumentMetadata>::new(y));
        assert!( matches!(
            dbg!(&b),
            Some(Ok(b_)) if b_.year() == &2026
        ) );
        assert!( matches!(
            dbg!(b.transpose()),
            Ok(Some(b_)) if b_.year() == &2026
        ) );

        let c = Some(1);
        let d = c.map(|y| Year::<DocumentMetadata>::new(y));
        assert!( matches!(
            dbg!(&d),
            Some(Err(_))
        ) );
        assert!( matches!(
            dbg!(d.transpose()),
            Err(_)
        ) );

        let e = None;
        let f = e.map(|y| Year::<DocumentMetadata>::new(y));
        assert!( matches!(
            dbg!(&f),
            None
        ) );
        assert!( matches!(
            dbg!(f.transpose()),
            Ok(None),
        ) );
    }
}
