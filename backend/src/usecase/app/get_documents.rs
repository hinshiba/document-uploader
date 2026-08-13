use crate::{
    domain::{
        Id,
        Year,
        document::{
            Document,
            DocumentMetadata,
            ExamType,
        },
        subject::Subject,
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
    pub subject_id: Id<Subject>,
    pub year: Option<Year<DocumentMetadata>>,
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
        let repo_option = SearchDocumentOption {
            subject_id: option.subject_id,
            year: option.year,
            teacher: option.teacher,
            exam_type: option.exam_type,
            is_answer: option.is_answer,
        };

        let result_vec = self.repository.search_documents(repo_option).await?;

        Ok(result_vec)
    }
}
