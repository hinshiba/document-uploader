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
    pub faculty_id: Id<Faculty>,
    pub major_id: Id<Major>,
    pub grade: Grade<Subject>,
    pub term: Term<Subject>,
}

#[derive(Debug, Hash)]
pub enum CreateSubjectOutput {
    Created(Subject),
    ErrCourseCodeNonUnique(String),
    ErrFacultyNotExist(Id<Faculty>),
    ErrMajorNotExist(Id<Major>),
    ErrFacultyMajorRelation(Id<Faculty>, Id<Major>),
}

impl<I> CreateSubjectUseCase<I> {
    pub fn new(repository: I) -> Self {
        Self { repository }
    }
}

impl<I: SubjectRepository> CreateSubjectUseCase<I> {
    // TODO: `course_code`による重複判定を実装する
    #[tracing::instrument(skip(self), ret(level="debug"), err)]
    pub async fn execute(&self, input: CreateSubjectInput) -> anyhow::Result<CreateSubjectOutput> {
        // SubjectIdはUseCaseで与える
        let subject_create_id: Id<Subject> = Id::new(uuid::Uuid::new_v4());

        let subject_create = Subject::new(
            subject_create_id.clone(),
            input.name,
            // TODO: `faculty_id`を持つFacultyが存在することを検証する
            input.faculty_id,
            // TODO: `major_id`を持つMajorが存在することを検証する
            input.major_id,
            input.grade,
            input.term,
        );

        // `Subject`の手動Clone
        let subject_ret = Subject::new(
            subject_create_id,
            subject_create.name().to_owned(),
            subject_create.faculty_id().clone(),
            subject_create.major_id().clone(),
            subject_create.grade().clone(),
            subject_create.term().clone(),
        );

        self.repository.create_subject(subject_create).await?;

        Ok(CreateSubjectOutput::Created(subject_ret))
    }
}
