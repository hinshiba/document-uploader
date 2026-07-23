use std::collections::HashMap;

use sqlx::PgPool;

use crate::{
    domain::{
        Grade, Id, Num, Term, Year,
        document::{Document, DocumentFile, DocumentMetadata, ExamType},
        faculty::Faculty,
        major::Major,
        subject::Subject,
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
    #[tracing::instrument(skip(self), ret, err(Display))]
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
    #[tracing::instrument(skip(self), ret, err(Display))]
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
    #[tracing::instrument(skip(self), err(Display))]
    async fn find_document_by_id(
        &self,
        document_id: &Id<Document>,
    ) -> anyhow::Result<Option<Document>> {
        let Some(row) = sqlx::query!(
            r#"
            SELECT
                d.id,
                d.year,
                d.teacher,
                d.exam_type,
                d.is_answer,
                d.num,
                s.id AS subject_id,
                s.major_id,
                s.grade,
                s.term,
                m.faculty_id
            FROM documents AS d
                INNER JOIN subjects AS s ON s.id = d.subject_id
                INNER JOIN majors AS m ON m.id = s.major_id
            WHERE d.id = $1
        "#,
            document_id.id(),
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        // 紐づくファイル情報を取得
        let files = sqlx::query!(
            r#"
            SELECT file_type, path
            FROM document_files
            WHERE document_id = $1
        "#,
            document_id.id(),
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|f| Ok(DocumentFile::new(f.file_type.parse()?, f.path.into())))
        .collect::<anyhow::Result<Vec<_>>>()?;

        let metadata = DocumentMetadata::new(
            Id::new(row.faculty_id),
            Id::new(row.major_id),
            Year::new(row.year)?,
            Term::new(row.term)?,
            Grade::new(row.grade)?,
            Id::new(row.subject_id),
            row.teacher,
            ExamType::from_int(row.exam_type)
                .ok_or_else(|| anyhow::anyhow!("Invalid exam_type stored in database."))?,
            row.is_answer,
            Num::new(row.num)?,
        );

        Ok(Some(Document::new(Id::new(row.id), metadata, files)?))
    }

    #[tracing::instrument(skip(self), err(Display))]
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
        let document_id = document.id().id();
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    /// 空のDBがあるか確認
    #[sqlx::test]
    async fn migrations_run(pool: PgPool) {
        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM faculties")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, Some(0));
    }

    // list_facultiesについて
    /// 専攻が0の要素の列挙可能性を確認
    #[sqlx::test]
    async fn list_faculties_groups_majors(pool: PgPool) {
        // 初期値の生成
        let eng_id = Uuid::new_v4();
        let sci_id = Uuid::new_v4();
        sqlx::query!(
            r#"
        INSERT INTO faculties (id, name)
            VALUES ($1, $2), ($3, $4)
        "#,
            eng_id,
            "工学部",
            sci_id,
            "理学部"
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO majors (id, name, faculty_id) 
                VALUES ($1, $2, $3), ($4, $5, $6)",
            Uuid::new_v4(),
            "情報工学コース",
            eng_id,
            Uuid::new_v4(),
            "ネットワーク工学コース",
            eng_id
        )
        .execute(&pool)
        .await
        .unwrap();

        // 実行
        let repo = PostgresRepository::new(pool);
        let faculties = repo.list_faculties().await.unwrap();

        assert_eq!(faculties.len(), 2);

        let eng_faculty = faculties
            .iter()
            .find(|f| f.id().id() == &eng_id)
            .expect("工学部なし");
        let mut major_names: Vec<_> = eng_faculty.majors().iter().map(|m| m.name()).collect();
        major_names.sort();
        assert_eq!(major_names, ["ネットワーク工学コース", "情報工学コース"]);

        let sci_faculty = faculties
            .iter()
            .find(|f| f.id().id() == &sci_id)
            .expect("理学部なし");
        assert!(sci_faculty.majors().is_empty());
    }

    // list_subjectsについて
    ///
    #[sqlx::test]
    async fn list_subjects_resolves_faculty_via_major(pool: PgPool) {
        // 初期値の生成
        let eng_id = Uuid::new_v4();
        let sci_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO faculties (id, name) 
                VALUES ($1, $2), ($3, $4)",
            eng_id,
            "工学部",
            sci_id,
            "理学部"
        )
        .execute(&pool)
        .await
        .unwrap();

        let eng_major = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO majors (id, name, faculty_id) 
                VALUES ($1, $2, $3), ($4, $5, $6)",
            eng_major,
            "情報工学コース",
            eng_id,
            Uuid::new_v4(),
            "数学科",
            sci_id
        )
        .execute(&pool)
        .await
        .unwrap();

        let subject_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO subjects (id, name, major_id, grade, term) 
                VALUES ($1, $2, $3, $4, $5)",
            subject_id,
            "線形代数",
            eng_major,
            1i64,
            2i64
        )
        .execute(&pool)
        .await
        .unwrap();

        // 実行
        let repo = PostgresRepository::new(pool);
        let subjects = repo.list_subjects().await.unwrap();

        assert_eq!(subjects.len(), 1);
        let subject = &subjects[0];
        assert_eq!(subject.id().id(), &subject_id);
        assert_eq!(subject.name(), "線形代数");
        assert_eq!(subject.major_id().id(), &eng_major);
        assert_eq!(subject.faculty_id().id(), &eng_id);
        assert_eq!(subject.grade().grade(), &1);
        assert_eq!(subject.term().term(), &2);
    }

    // find_document_by_idについて
    /// subjects,majorsをjoinしてメタデータ・ファイルを復元できるか確認
    #[sqlx::test]
    async fn find_document_by_id_reconstructs_document(pool: PgPool) {
        // 初期値の生成
        let faculty_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO faculties (id, name) VALUES ($1, $2)",
            faculty_id,
            "工学部"
        )
        .execute(&pool)
        .await
        .unwrap();

        let major_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO majors (id, name, faculty_id) VALUES ($1, $2, $3)",
            major_id,
            "情報工学コース",
            faculty_id
        )
        .execute(&pool)
        .await
        .unwrap();

        let subject_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO subjects (id, name, major_id, grade, term)
                VALUES ($1, $2, $3, $4, $5)",
            subject_id,
            "線形代数",
            major_id,
            1i64,
            2i64
        )
        .execute(&pool)
        .await
        .unwrap();

        let document_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO documents (id, subject_id, year, teacher, exam_type, is_answer, num)
                VALUES ($1, $2, $3, $4, $5, $6, $7)",
            document_id,
            subject_id,
            2024i64,
            "山田",
            ExamType::FinalTerm.to_int(),
            false,
            1i64
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO document_files (document_id, file_type, path)
                VALUES ($1, $2, $3), ($4, $5, $6)",
            document_id,
            "pdf",
            "path/to/a.pdf",
            document_id,
            "jpeg",
            "path/to/b.jpg"
        )
        .execute(&pool)
        .await
        .unwrap();

        // 実行
        let repo = PostgresRepository::new(pool);
        let document = repo
            .find_document_by_id(&Id::new(document_id))
            .await
            .unwrap()
            .expect("ドキュメントなし");

        assert_eq!(document.id().id(), &document_id);

        let meta = document.metadata();
        assert_eq!(meta.faculty_id().id(), &faculty_id);
        assert_eq!(meta.major_id().id(), &major_id);
        assert_eq!(meta.subject_id().id(), &subject_id);
        assert_eq!(meta.year().year(), &2024);
        assert_eq!(meta.term().term(), &2);
        assert_eq!(meta.grade().grade(), &1);
        assert_eq!(meta.teacher(), "山田");
        assert_eq!(meta.exam_type(), &ExamType::FinalTerm);
        assert_eq!(meta.is_answer(), &false);
        assert_eq!(meta.num().num(), &1);

        let mut paths: Vec<_> = document
            .files()
            .iter()
            .map(|f| f.path().to_str().unwrap())
            .collect();
        paths.sort();
        assert_eq!(paths, ["path/to/a.pdf", "path/to/b.jpg"]);
    }

    /// 存在しないidではNoneを返すか
    #[sqlx::test]
    async fn find_document_by_id_returns_none_when_missing(pool: PgPool) {
        let repo = PostgresRepository::new(pool);
        let document = repo
            .find_document_by_id(&Id::new(Uuid::new_v4()))
            .await
            .unwrap();
        assert!(document.is_none());
    }
}
