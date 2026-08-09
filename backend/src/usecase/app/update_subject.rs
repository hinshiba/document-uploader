use crate::domain::{
    Id,
    Grade,
    Term,
    faculty::Faculty,
    subject::Subject,
    major::Major,
};
use crate::usecase::repository::{
    SubjectRepository,
    SearchSubjectOption,
    UpdateSubjectContent,
};

#[derive(Debug)]
pub struct UpdateSubjectUseCase<I> {
    repository: I
}

#[derive(Debug, Clone, Hash)]
pub struct UpdateSubjectInput {
    pub id: Id<Subject>,
    pub name: String,
    pub course_code: String,
    pub faculty_id: Id<Faculty>,
    pub major_id: Id<Major>,
    pub grade: Grade<Subject>,
    pub term: Term<Subject>,
}

#[derive(Debug, Clone, Hash)]
pub enum UpdateSubjectOutput {
    Updated,
    ErrCourseCodeNonUnique(String),
    ErrFacultyNotExist(Id<Faculty>),
    ErrMajorNotExist(Id<Major>),
    ErrSubjectNotExist(Id<Subject>),
    ErrFacultyMajorRelation(Id<Faculty>, Id<Major>),
}

impl<I> UpdateSubjectUseCase<I> {
    pub fn new(repository: I) -> Self {
        Self { repository }
    }
}

impl<I: SubjectRepository> UpdateSubjectUseCase<I> {
    // TODO: `course_code`と`major_id`, `faculty_id`の検証を追加する
    #[tracing::instrument(skip(self), ret(level="debug"), err)]
    pub async fn execute(&self, input: UpdateSubjectInput) -> anyhow::Result<UpdateSubjectOutput> {
        // 変更対象となる`Subject`が存在するか検証する
        if self.repository.search_subjects(
                SearchSubjectOption {
                    subject_id: Some(input.id.clone()),
                    ..Default::default()
                }
            ).await?
            .is_empty()
        {
            return Ok(UpdateSubjectOutput::ErrSubjectNotExist(input.id))
        };

        let content = UpdateSubjectContent {
            name: input.name,
            faculty_id: input.faculty_id,
            major_id: input.major_id,
            grade: input.grade,
            term: input.term,
        };

        self.repository.update_subject(input.id, content).await?;

        Ok(UpdateSubjectOutput::Updated)
    }
}
