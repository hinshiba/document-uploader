use std::collections::HashMap;

use sqlx::{PgPool, query};
use uuid::Uuid;

use crate::{
    domain::{
        Grade, Id, Term, document::Document, faculty::Faculty, major::Major, subject::Subject,
    },
    usecase::repository::{DocumentRepository, FacultyRepository, SubjectRepository},
};

#[derive(Debug)]
pub struct PostgresRepository {
    pool: sqlx::PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl SubjectRepository for PostgresRepository {
    #[tracing::instrument(skip(self), err(Debug))]
    async fn list_subjects(&self) -> anyhow::Result<Vec<Subject>> {
        // subjects, majorsから必要な情報を取得
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

impl FacultyRepository for PostgresRepository {
    #[tracing::instrument(skip(self), err(Debug))]
    async fn list_faculties(&self) -> anyhow::Result<Vec<Faculty>> {
        // 学部一覧を取得
        let faculties = sqlx::query!(
            r#"
            SELECT id, name
            FROM faculties
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        // 専攻一覧を取得
        let majors = sqlx::query!(
            r#"
            SELECT id, name, faculty_id
            FROM majors
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        // 学部と専攻の対応を作成
        let mut major_map: HashMap<uuid::Uuid, Vec<Major>> = HashMap::new();
        for m in majors {
            major_map.entry(m.faculty_id).or_default().push(Major::new(
                Id::new(m.id),
                m.name,
                Id::new(m.faculty_id),
            ))
        }

        // mapをremoveしながら生成
        Ok(faculties
            .into_iter()
            .map(|f| {
                Faculty::new(
                    Id::new(f.id),
                    f.name,
                    major_map.remove(&f.id).unwrap_or_default(),
                )
            })
            .collect())
    }
}

impl DocumentRepository for PostgresRepository {
    #[tracing::instrument(skip(self), err(Debug))]
    async fn store_document(&self, document: Document) -> anyhow::Result<()> {
        let mut transaction = self.pool.begin().await?;

        let meta = document.metadata();

        // 存在しない学部等idではないか確認
        let _ = sqlx::query!(
            r#"
            SELECT s.id
            FROM subjects AS s
            INNER JOIN majors AS m ON m.id = s.major_id
            WHERE
                s.id = $1 AND
                m.faculty_id = $2 AND
                s.major_id = $3 AND
                s.grade = $4 AND
                s.term = $5
        "#,
            meta.subject_id().id(),
            meta.faculty_id().id(),
            meta.major_id().id(),
            meta.grade().grade(),
            meta.term().term()
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| anyhow::anyhow!("No subject matching the specified criteria was found."))?;

        // メタデータの格納
        let document_id = Uuid::new_v4();
        let _ = sqlx::query!(
            r#"
            INSERT INTO documents (id, subject_id, year, teacher, exam_type, is_answer, num)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
            document_id,
            meta.subject_id().id(),
            meta.year().year(),
            meta.teacher(),
            meta.exam_type().to_int(),
            meta.is_answer(),
            meta.num().num(),
        )
        .execute(&mut *transaction)
        .await?;

        // ファイル情報の格納
        for file in document.files() {
            let path = file
                .path()
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("File path is not valid UTF-8."))?;

            let _ = sqlx::query!(
                r#"
                INSERT INTO document_files (document_id, file_type, path)
                    VALUES ($1, $2, $3)
            "#,
                document_id,
                file.ty().to_string(),
                path,
            )
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }
}
