use sqlx::PgPool;

use crate::{
    domain::{Grade, Id, Term, subject::Subject},
    usecase::repository::SubjectRepository,
};

#[derive(Debug)]
pub struct SqlxRepository {
    pool: sqlx::PgPool,
}

impl SqlxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl SubjectRepository for SqlxRepository {
    #[tracing::instrument(skip(self), err(Debug))]
    async fn list_subjects(&self) -> anyhow::Result<Vec<Subject>> {
        sqlx::query!(
            r#"
            SELECT s.id, s.name, m.faculty_id, s.major_id, s.grade, s.term
            FROM subjects AS s
            INNER JOIN majors AS m ON m.id = s.major_id
        "#
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|r| {
            Ok(Subject::new(
                Id::new(r.id),
                r.name,
                Id::new(r.faculty_id),
                Id::new(r.major_id),
                Grade::new(r.grade)?,
                Term::new(r.term)?,
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()
    }
}
