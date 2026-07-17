use crate::domain::subject::Subject;
use crate::usecase::repository::SubjectRepository;

#[derive(Debug)]
pub struct GetSubjectsUseCase<I> {
    repository: I
}

#[derive(Debug, Clone, Hash)]
pub struct GetSubjectsOption {
    pub faculty_id: uuid::Uuid,
    pub major_id: Option<uuid::Uuid>,
    pub grade: Option<i64>,
    pub term: Option<i64>,
}

impl<I> GetSubjectsUseCase<I> {
    pub fn new(repository: I) -> Self {
        Self { repository }
    }
}

impl<I: SubjectRepository> GetSubjectsUseCase<I> {
    #[tracing::instrument(skip(self), ret(level="debug"), err)]
    pub async fn execute(&self, option: GetSubjectsOption) -> anyhow::Result<Vec<Subject>> {
        let subjects = self.repository.list_subjects().await?;

        // FIXME: 登録されている科目が増えてくると、アプリケーションの処理が重くなる
        let subjects = subjects
            .into_iter()
            .filter(|subject| {
                   &option.faculty_id == subject.faculty_id().id()
                && option.major_id.map_or(true, |inner| &inner == subject.major_id().id())
                && option.grade.map_or(true, |inner| &inner == subject.grade().grade())
                && option.term.map_or(true, |inner| &inner == subject.term().term())
            })
            .collect();

        Ok(subjects)
    }
}
