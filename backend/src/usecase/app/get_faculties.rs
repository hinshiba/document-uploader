use crate::domain::faculty::Faculty;
use crate::usecase::repository::FacultyRepository;

#[derive(Debug)]
pub struct GetFacultiesUseCase<I> {
    repository: I
}

impl<I> GetFacultiesUseCase<I> {
    pub fn new(repository: I) -> Self {
        Self { repository }
    }
}

impl<I: FacultyRepository> GetFacultiesUseCase<I> {
    #[tracing::instrument(skip(self), ret(level="debug"), err)]
    pub async fn execute(&self) -> anyhow::Result<Vec<Faculty>> {
        self.repository.list_faculties().await
    }
}
