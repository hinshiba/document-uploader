use crate::domain::{
    Id,
    Grade,
    Term,
    faculty::Faculty,
    subject::Subject,
    major::Major,
};
use crate::usecase::repository::SubjectRepository;

#[derive(Debug)]
pub struct CreateSubjectUseCase<I> {
    repository: I
}

#[derive(Debug, Clone, Hash)]
pub struct CreateSubjectInput {
    pub name: String,
    pub course_code: String,
    pub faculty_id: uuid::Uuid,
    pub major_id: uuid::Uuid,
    pub grade: Grade<Subject>,
    pub term: Term<Subject>,
}

#[derive(Debug, Clone, Hash)]
pub enum CreateSubjectOutput {
    Created,
    ErrCourseCodeNonUnique(String),
    ErrFacultyNotExist(Id<Faculty>),
    ErrSubjectNotExist(Id<Subject>),
    ErrMajorNotExist(Id<Major>),
}

impl<I> CreateSubjectUseCase<I> {
    pub fn new(repository: I) -> Self {
        Self { repository }
    }
}

impl<I: SubjectRepository> CreateSubjectUseCase<I> {
    // TODO: `course_code`による重複判定を実装する
    #[tracing::instrument(skip(self), err)]
    pub async fn execute(&self, input: CreateSubjectInput) -> anyhow::Result<CreateSubjectOutput> {
        // SubjectIdはUseCaseで与える
        let subject_create_id: Id<Subject> = Id::new(uuid::Uuid::new_v4());

        let subject_create = Subject::new(
            subject_create_id,
            input.name,
            // TODO: `faculty_id`を持つFacultyが存在することを検証する
            Id::new(input.faculty_id),
            // TODO: `major_id`を持つMajorが存在することを検証する
            Id::new(input.major_id),
            input.grade,
            input.term,
        );

        self.repository.create_subject(subject_create).await?;

        Ok(CreateSubjectOutput::Created)
    }
}
