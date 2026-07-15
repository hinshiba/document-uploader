use std::collections::HashMap;

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::{
    Grade,
    Id,
    Term,
    document::{Document, DocumentFile, DocumentFileType},
    faculty::Faculty,
    major::Major,
    subject::Subject,
};
use crate::usecase::repository::{
    DocumentFileRepository,
    DocumentRepository,
    FacultyRepository,
    SubjectRepository,
};

/// sqlx(SQLite)ベースのRepository実装．
///
/// `SqlitePool`は内部的にArc共有されるためcloneは安価．
#[derive(Clone, Debug)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl FacultyRepository for SqliteRepository {
    #[tracing::instrument(skip(self), err)]
    async fn list_faculties(&self) -> anyhow::Result<Vec<Faculty>> {
        let major_rows = sqlx::query("SELECT id, name, faculty_id FROM majors ORDER BY id")
            .fetch_all(&self.pool)
            .await?;

        let mut majors_by_faculty: HashMap<String, Vec<Major>> = HashMap::new();
        for row in major_rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let faculty_id: String = row.get("faculty_id");

            let major = Major::new(
                Id::new(Uuid::parse_str(&id)?),
                name,
                Id::new(Uuid::parse_str(&faculty_id)?),
            );
            majors_by_faculty.entry(faculty_id).or_default().push(major);
        }

        let faculty_rows = sqlx::query("SELECT id, name FROM faculties ORDER BY id")
            .fetch_all(&self.pool)
            .await?;

        let mut faculties = Vec::with_capacity(faculty_rows.len());
        for row in faculty_rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let majors = majors_by_faculty.remove(&id).unwrap_or_default();

            faculties.push(Faculty::new(Id::new(Uuid::parse_str(&id)?), name, majors));
        }

        Ok(faculties)
    }
}

impl SubjectRepository for SqliteRepository {
    #[tracing::instrument(skip(self), err)]
    async fn list_subjects(&self) -> anyhow::Result<Vec<Subject>> {
        let rows =
            sqlx::query("SELECT id, name, faculty_id, major_id, grade, term FROM subjects ORDER BY id")
                .fetch_all(&self.pool)
                .await?;

        let mut subjects = Vec::with_capacity(rows.len());
        for row in rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let faculty_id: String = row.get("faculty_id");
            let major_id: String = row.get("major_id");
            let grade: i64 = row.get("grade");
            let term: i64 = row.get("term");

            subjects.push(Subject::new(
                Id::new(Uuid::parse_str(&id)?),
                name,
                Id::new(Uuid::parse_str(&faculty_id)?),
                Id::new(Uuid::parse_str(&major_id)?),
                Grade::new(grade)?,
                Term::new(term)?,
            ));
        }

        Ok(subjects)
    }
}

impl DocumentFileRepository for SqliteRepository {
    #[tracing::instrument(skip_all)]
    async fn store_document_file(
        &self,
        content: Vec<u8>,
        file_type: DocumentFileType,
    ) -> anyhow::Result<DocumentFile> {
        // ハッカソン方針: アップロードされたファイルは永続化せず破棄する．
        let _discarded = content;
        tracing::info!(?file_type, "received a document file (discarded, not persisted)");

        Ok(DocumentFile::new(file_type, std::path::PathBuf::new()))
    }
}

impl DocumentRepository for SqliteRepository {
    #[tracing::instrument(skip_all)]
    async fn store_document(&self, document: Document) -> anyhow::Result<()> {
        // ハッカソン方針: ドキュメントも永続化しない．メタデータのパース確認のみ行う．
        tracing::info!(metadata = ?document.metadata(), "received a document (accepted, not persisted)");
        Ok(())
    }
}
