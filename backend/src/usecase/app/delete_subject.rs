use crate::domain::{
    Id,
    subject::Subject,
};
use crate::usecase::repository::SubjectRepository;

#[derive(Debug)]
pub struct DeleteSubjectUseCase<I> {
    repository: I
}

#[derive(Debug, Clone, Hash)]
pub struct DeleteSubjectInput {
    pub subject_id: Id<Subject>,
}

#[derive(Debug, Clone, Hash)]
pub enum DeleteSubjectOutput {
    Deleted,
    ErrSubjectNotExist(Id<Subject>),
    ErrReferencedByDocuments,
}

impl<I> DeleteSubjectUseCase<I> {
    pub fn new(repository: I) -> Self {
        Self { repository }
    }
}

impl<I: SubjectRepository> DeleteSubjectUseCase<I> {
    // TODO: SubjectIdがドキュメントから参照されているかを検証する
    #[tracing::instrument(skip(self), ret(level="debug"), err)]
    pub async fn execute(&self, input: DeleteSubjectInput) -> anyhow::Result<DeleteSubjectOutput> {
        self.repository.delete_subject(input.subject_id).await?;

        Ok(DeleteSubjectOutput::Deleted)
    }
}
