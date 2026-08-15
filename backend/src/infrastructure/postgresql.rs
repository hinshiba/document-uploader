use std::collections::HashMap;

use sqlx::PgPool;

use crate::{
    domain::{
        Grade, Id, Num, Term, Year,
        document::{Document, DocumentFile, DocumentMetadata, ExamType},
        faculty::Faculty,
        major::Major,
        subject::{CourseCode, Subject},
    },
    usecase::repository::{
        DocumentRepository, FacultyRepository, SearchSubjectOption, SubjectRepository,
        UpdateSubjectContent,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresRepository {
    pool: sqlx::PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 指定された専攻が指定された学部に属することを確認する
    async fn check_major_faculty_relation(
        executor: impl sqlx::PgExecutor<'_>,
        major_id: &Id<Major>,
        faculty_id: &Id<Faculty>,
    ) -> anyhow::Result<()> {
        let _ = sqlx::query!(
            r#"
            SELECT id
            FROM majors
            WHERE id = $1 AND faculty_id = $2
        "#,
            major_id.id(),
            faculty_id.id(),
        )
        .fetch_optional(executor)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!("The specified major does not belong to the specified faculty.")
        })?;

        Ok(())
    }
}

impl SubjectRepository for PostgresRepository {
    #[tracing::instrument(skip(self), ret, err(Display))]
    async fn list_subjects(&self) -> anyhow::Result<Vec<Subject>> {
        // 条件を1つも指定しない検索と等価
        self.search_subjects(SearchSubjectOption::default()).await
    }

    #[tracing::instrument(skip(self), ret, err(Display))]
    async fn search_subjects(&self, option: SearchSubjectOption) -> anyhow::Result<Vec<Subject>> {
        // NULLの条件は絞り込みに寄与させない
        sqlx::query!(
            r#"
            SELECT
                id AS "id!", name AS "name!", course_code AS "course_code!", faculty_id AS "faculty_id!",
                major_id AS "major_id!", grade AS "grade!", term AS "term!"
            FROM subject_details
            WHERE
                ($1::uuid IS NULL OR id = $1) AND
                ($2::text IS NULL OR name = $2) AND
                ($3::uuid IS NULL OR faculty_id = $3) AND
                ($4::uuid IS NULL OR major_id = $4) AND
                ($5::bigint IS NULL OR grade = $5) AND
                ($6::bigint IS NULL OR term = $6)
        "#,
            option.subject_id.as_ref().map(|id| *id.id()),
            option.name.as_deref(),
            option.faculty_id.as_ref().map(|id| *id.id()),
            option.major_id.as_ref().map(|id| *id.id()),
            option.grade.map(|grade| *grade.grade()),
            option.term.map(|term| *term.term()),
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|r| {
            Ok(Subject::new(
                Id::new(r.id),
                r.name,
                CourseCode::new(r.course_code)?,
                Id::new(r.faculty_id),
                Id::new(r.major_id),
                Grade::new(r.grade)?,
                Term::new(r.term)?,
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()
    }

    #[tracing::instrument(skip(self), err(Display))]
    async fn create_subject(&self, subject: Subject) -> anyhow::Result<()> {
        let mut transaction = self.pool.begin().await?;

        Self::check_major_faculty_relation(
            &mut *transaction,
            subject.major_id(),
            subject.faculty_id(),
        )
        .await?;

        // 科目の格納
        let result = sqlx::query!(
            r#"
            INSERT INTO subjects (id, name, course_code, major_id, grade, term)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (id) DO NOTHING
        "#,
            subject.id().id(),
            subject.name(),
            subject.course_code().code(),
            subject.major_id().id(),
            subject.grade().grade(),
            subject.term().term(),
        )
        .execute(&mut *transaction)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("subject already exists"));
        }

        transaction.commit().await?;

        Ok(())
    }

    #[tracing::instrument(skip(self), ret, err(Display))]
    async fn update_subject(
        &self,
        subject_id: Id<Subject>,
        content: UpdateSubjectContent,
    ) -> anyhow::Result<Subject> {
        let mut transaction = self.pool.begin().await?;

        Self::check_major_faculty_relation(
            &mut *transaction,
            &content.major_id,
            &content.faculty_id,
        )
        .await?;

        // 更新
        let updated = sqlx::query!(
            r#"
            UPDATE subjects
                SET name = $2, course_code = $3, major_id = $4, grade = $5, term = $6
                WHERE id = $1
                RETURNING id, name, course_code, major_id, grade, term
        "#,
            subject_id.id(),
            content.name,
            content.course_code.code(),
            content.major_id.id(),
            content.grade.grade(),
            content.term.term(),
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| anyhow::anyhow!("subject not found"))?;

        transaction.commit().await?;

        Ok(Subject::new(
            Id::new(updated.id),
            updated.name,
            CourseCode::new(updated.course_code)?,
            content.faculty_id,
            Id::new(updated.major_id),
            Grade::new(updated.grade)?,
            Term::new(updated.term)?,
        ))
    }

    #[tracing::instrument(skip(self), ret, err(Display))]
    async fn delete_subject(&self, subject_id: Id<Subject>) -> anyhow::Result<Subject> {
        let deleted = sqlx::query!(
            r#"
            DELETE FROM subjects AS s
                USING majors AS m
                WHERE s.major_id = m.id AND s.id = $1
                RETURNING s.id, s.name, s.course_code, m.faculty_id, s.major_id, s.grade, s.term
        "#,
            subject_id.id(),
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("subject does not exist"))?;

        Ok(Subject::new(
            Id::new(deleted.id),
            deleted.name,
            CourseCode::new(deleted.course_code)?,
            Id::new(deleted.faculty_id),
            Id::new(deleted.major_id),
            Grade::new(deleted.grade)?,
            Term::new(deleted.term)?,
        ))
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
            "INSERT INTO subjects (id, name, course_code, major_id, grade, term)
                VALUES ($1, $2, $3, $4, $5, $6)",
            subject_id,
            "線形代数",
            "090219",
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
        assert_eq!(subject.course_code().code(), "090219");
        assert_eq!(subject.major_id().id(), &eng_major);
        assert_eq!(subject.faculty_id().id(), &eng_id);
        assert_eq!(subject.grade().grade(), &1);
        assert_eq!(subject.term().term(), &2);
    }

    // search_subjectsについて
    /// 条件を1つも指定しないとき全件返ることを確認
    #[sqlx::test]
    async fn search_subjects_returns_all_when_option_is_empty(pool: PgPool) {
        let (_, eng_major, _, sci_major) = seed_faculties_and_majors(&pool).await;
        insert_subject(&pool, Uuid::new_v4(), "線形代数", "C001", eng_major, 1, 1).await;
        insert_subject(&pool, Uuid::new_v4(), "解析学", "C002", sci_major, 2, 3).await;

        let repo = PostgresRepository::new(pool);
        let subjects = repo
            .search_subjects(SearchSubjectOption::default())
            .await
            .unwrap();

        assert_eq!(subjects.len(), 2);
    }

    /// majors経由の学部絞り込みが効くことを確認
    #[sqlx::test]
    async fn search_subjects_filters_by_faculty(pool: PgPool) {
        let (eng_id, eng_major, _, sci_major) = seed_faculties_and_majors(&pool).await;
        let eng_subject = Uuid::new_v4();
        insert_subject(&pool, eng_subject, "線形代数", "C001", eng_major, 1, 1).await;
        insert_subject(&pool, Uuid::new_v4(), "解析学", "C002", sci_major, 2, 3).await;

        let repo = PostgresRepository::new(pool);
        let subjects = repo
            .search_subjects(SearchSubjectOption {
                faculty_id: Some(Id::new(eng_id)),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].id().id(), &eng_subject);
        assert_eq!(subjects[0].faculty_id().id(), &eng_id);
    }

    /// 複数条件がANDで結合されることを確認
    #[sqlx::test]
    async fn search_subjects_filters_by_grade_and_term(pool: PgPool) {
        let (_, eng_major, _, _) = seed_faculties_and_majors(&pool).await;
        let target = Uuid::new_v4();
        insert_subject(&pool, target, "線形代数", "C001", eng_major, 2, 3).await;
        // 学年のみ一致
        insert_subject(&pool, Uuid::new_v4(), "電磁気学", "C002", eng_major, 2, 1).await;
        // 学期のみ一致
        insert_subject(&pool, Uuid::new_v4(), "熱力学", "C003", eng_major, 1, 3).await;

        let repo = PostgresRepository::new(pool);
        let subjects = repo
            .search_subjects(SearchSubjectOption {
                grade: Some(Grade::new(2).unwrap()),
                term: Some(Term::new(3).unwrap()),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].id().id(), &target);
    }

    // create_subjectについて
    /// 作成した科目が学部込みで読み戻せることを確認
    #[sqlx::test]
    async fn create_subject_inserts_row(pool: PgPool) {
        let (eng_id, eng_major, _, _) = seed_faculties_and_majors(&pool).await;
        let subject_id = Uuid::new_v4();

        let repo = PostgresRepository::new(pool);
        repo.create_subject(subject_of(subject_id, "線形代数", "C001", eng_id, eng_major, 1, 2))
            .await
            .unwrap();

        let subjects = repo
            .search_subjects(SearchSubjectOption {
                subject_id: Some(Id::new(subject_id)),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(subjects.len(), 1);
        let subject = &subjects[0];
        assert_eq!(subject.name(), "線形代数");
        assert_eq!(subject.course_code().code(), "C001");
        assert_eq!(subject.faculty_id().id(), &eng_id);
        assert_eq!(subject.major_id().id(), &eng_major);
        assert_eq!(subject.grade().grade(), &1);
        assert_eq!(subject.term().term(), &2);
    }

    /// id重複がエラーとなり, 既存の行が書き換わらないことを確認
    #[sqlx::test]
    async fn create_subject_rejects_duplicate_id(pool: PgPool) {
        let (eng_id, eng_major, _, _) = seed_faculties_and_majors(&pool).await;
        let subject_id = Uuid::new_v4();

        let repo = PostgresRepository::new(pool);
        repo.create_subject(subject_of(subject_id, "線形代数", "C001", eng_id, eng_major, 1, 2))
            .await
            .unwrap();

        let result = repo
            .create_subject(subject_of(subject_id, "解析学", "C002", eng_id, eng_major, 3, 4))
            .await;
        assert!(result.is_err());

        // 既存の行が保持されていること
        let subjects = repo
            .search_subjects(SearchSubjectOption {
                subject_id: Some(Id::new(subject_id)),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(subjects[0].name(), "線形代数");
    }

    /// 専攻が学部に属さない組み合わせが弾かれることを確認
    #[sqlx::test]
    async fn create_subject_rejects_major_faculty_mismatch(pool: PgPool) {
        let (eng_id, _, _, sci_major) = seed_faculties_and_majors(&pool).await;

        let repo = PostgresRepository::new(pool);
        // 工学部に数学科を組み合わせる
        let result = repo
            .create_subject(subject_of(
                Uuid::new_v4(),
                "線形代数",
                "C001",
                eng_id,
                sci_major,
                1,
                2,
            ))
            .await;

        assert!(result.is_err());

        let subjects = repo
            .search_subjects(SearchSubjectOption::default())
            .await
            .unwrap();
        assert!(subjects.is_empty());
    }

    // update_subjectについて
    /// 更新後の値が返り, DBにも反映されることを確認
    #[sqlx::test]
    async fn update_subject_returns_updated_subject(pool: PgPool) {
        let (eng_id, eng_major, sci_id, sci_major) = seed_faculties_and_majors(&pool).await;
        let subject_id = Uuid::new_v4();
        insert_subject(&pool, subject_id, "線形代数", "C001", eng_major, 1, 2).await;

        let repo = PostgresRepository::new(pool);
        let updated = repo
            .update_subject(
                Id::new(subject_id),
                UpdateSubjectContent {
                    name: "解析学".to_owned(),
                    course_code: CourseCode::new("C002").unwrap(),
                    faculty_id: Id::new(sci_id),
                    major_id: Id::new(sci_major),
                    grade: Grade::new(3).unwrap(),
                    term: Term::new(4).unwrap(),
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.id().id(), &subject_id);
        assert_eq!(updated.name(), "解析学");
        assert_eq!(updated.course_code().code(), "C002");
        assert_eq!(updated.faculty_id().id(), &sci_id);
        assert_eq!(updated.major_id().id(), &sci_major);
        assert_eq!(updated.grade().grade(), &3);
        assert_eq!(updated.term().term(), &4);

        // 読み戻してもfaculty_idが一致すること
        let stored = repo
            .search_subjects(SearchSubjectOption {
                subject_id: Some(Id::new(subject_id)),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(stored[0].faculty_id().id(), &sci_id);
        assert_ne!(stored[0].faculty_id().id(), &eng_id);
    }

    /// 存在しないidの更新がエラーとなることを確認
    #[sqlx::test]
    async fn update_subject_errors_when_not_found(pool: PgPool) {
        let (eng_id, eng_major, _, _) = seed_faculties_and_majors(&pool).await;

        let repo = PostgresRepository::new(pool);
        let result = repo
            .update_subject(
                Id::new(Uuid::new_v4()),
                UpdateSubjectContent {
                    name: "線形代数".to_owned(),
                    course_code: CourseCode::new("C001").unwrap(),
                    faculty_id: Id::new(eng_id),
                    major_id: Id::new(eng_major),
                    grade: Grade::new(1).unwrap(),
                    term: Term::new(2).unwrap(),
                },
            )
            .await;

        assert!(result.is_err());
    }

    /// 専攻が学部に属さない更新が弾かれ, 既存の行が変わらないことを確認
    #[sqlx::test]
    async fn update_subject_rejects_major_faculty_mismatch(pool: PgPool) {
        let (eng_id, eng_major, _, sci_major) = seed_faculties_and_majors(&pool).await;
        let subject_id = Uuid::new_v4();
        insert_subject(&pool, subject_id, "線形代数", "C001", eng_major, 1, 2).await;

        let repo = PostgresRepository::new(pool);
        let result = repo
            .update_subject(
                Id::new(subject_id),
                UpdateSubjectContent {
                    name: "解析学".to_owned(),
                    course_code: CourseCode::new("C002").unwrap(),
                    faculty_id: Id::new(eng_id),
                    major_id: Id::new(sci_major),
                    grade: Grade::new(3).unwrap(),
                    term: Term::new(4).unwrap(),
                },
            )
            .await;

        assert!(result.is_err());

        let stored = repo
            .search_subjects(SearchSubjectOption {
                subject_id: Some(Id::new(subject_id)),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(stored[0].name(), "線形代数");
    }

    // delete_subjectについて
    /// 削除前の値が返り, 行が消えることを確認
    #[sqlx::test]
    async fn delete_subject_returns_deleted_subject(pool: PgPool) {
        let (eng_id, eng_major, _, _) = seed_faculties_and_majors(&pool).await;
        let subject_id = Uuid::new_v4();
        insert_subject(&pool, subject_id, "線形代数", "C001", eng_major, 1, 2).await;

        let repo = PostgresRepository::new(pool);
        let deleted = repo.delete_subject(Id::new(subject_id)).await.unwrap();

        assert_eq!(deleted.id().id(), &subject_id);
        assert_eq!(deleted.name(), "線形代数");
        assert_eq!(deleted.course_code().code(), "C001");
        assert_eq!(deleted.faculty_id().id(), &eng_id);
        assert_eq!(deleted.major_id().id(), &eng_major);
        assert_eq!(deleted.grade().grade(), &1);
        assert_eq!(deleted.term().term(), &2);

        let subjects = repo
            .search_subjects(SearchSubjectOption::default())
            .await
            .unwrap();
        assert!(subjects.is_empty());
    }

    /// 存在しないidの削除がエラーとなることを確認
    #[sqlx::test]
    async fn delete_subject_errors_when_not_found(pool: PgPool) {
        let repo = PostgresRepository::new(pool);
        let result = repo.delete_subject(Id::new(Uuid::new_v4())).await;

        assert!(result.is_err());
    }

    /// documentsから参照されている科目が削除できないことを確認
    #[sqlx::test]
    async fn delete_subject_errors_when_referenced_by_document(pool: PgPool) {
        let (_, eng_major, _, _) = seed_faculties_and_majors(&pool).await;
        let subject_id = Uuid::new_v4();
        insert_subject(&pool, subject_id, "線形代数", "C001", eng_major, 1, 2).await;

        sqlx::query!(
            "INSERT INTO documents (id, subject_id, year, teacher, exam_type, is_answer, num)
                VALUES ($1, $2, $3, $4, $5, $6, $7)",
            Uuid::new_v4(),
            subject_id,
            2025i64,
            "山田",
            0i64,
            false,
            1i64
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = PostgresRepository::new(pool);
        let result = repo.delete_subject(Id::new(subject_id)).await;

        // 外部キー違反が伝播すること
        assert!(result.is_err());

        // ロールバックされ科目が残っていること
        let subjects = repo
            .search_subjects(SearchSubjectOption::default())
            .await
            .unwrap();
        assert_eq!(subjects.len(), 1);
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
            "INSERT INTO subjects (id, name, course_code, major_id, grade, term)
                VALUES ($1, $2, $3, $4, $5, $6)",
            subject_id,
            "線形代数",
            "C001",
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

    // 以下helper functions

    /// 工学部/情報工学コースと理学部/数学科を投入する
    ///
    /// 戻り値は`(工学部id, 情報工学コースid, 理学部id, 数学科id)`
    async fn seed_faculties_and_majors(pool: &PgPool) -> (Uuid, Uuid, Uuid, Uuid) {
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
        .execute(pool)
        .await
        .unwrap();

        let eng_major = Uuid::new_v4();
        let sci_major = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO majors (id, name, faculty_id)
                VALUES ($1, $2, $3), ($4, $5, $6)",
            eng_major,
            "情報工学コース",
            eng_id,
            sci_major,
            "数学科",
            sci_id
        )
        .execute(pool)
        .await
        .unwrap();

        (eng_id, eng_major, sci_id, sci_major)
    }

    fn subject_of(
        id: Uuid,
        name: &str,
        course_code: &str,
        faculty_id: Uuid,
        major_id: Uuid,
        grade: i64,
        term: i64,
    ) -> Subject {
        Subject::new(
            Id::new(id),
            name.to_owned(),
            CourseCode::new(course_code).unwrap(),
            Id::new(faculty_id),
            Id::new(major_id),
            Grade::new(grade).unwrap(),
            Term::new(term).unwrap(),
        )
    }

    /// `subjects`へ直接投入する, faculty_idはmajor_id経由でしか解決されない
    async fn insert_subject(
        pool: &PgPool,
        id: Uuid,
        name: &str,
        course_code: &str,
        major_id: Uuid,
        grade: i64,
        term: i64,
    ) {
        sqlx::query!(
            "INSERT INTO subjects (id, name, course_code, major_id, grade, term)
                VALUES ($1, $2, $3, $4, $5, $6)",
            id,
            name,
            course_code,
            major_id,
            grade,
            term
        )
        .execute(pool)
        .await
        .unwrap();
    }
}
