use crate::{
    domain::{Grade, Id, Term, faculty::Faculty, major::Major, subject::Subject},
    usecase::repository::{SearchSubjectOption, SubjectRepository, UpdateSubjectContent},
};

use super::PostgresRepository;

impl PostgresRepository {
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
                id AS "id!", name AS "name!", faculty_id AS "faculty_id!",
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
            INSERT INTO subjects (id, name, major_id, grade, term)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (id) DO NOTHING
        "#,
            subject.id().id(),
            subject.name(),
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
                SET name = $2, major_id = $3, grade = $4, term = $5
                WHERE id = $1
                RETURNING id, name, major_id, grade, term
        "#,
            subject_id.id(),
            content.name,
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
                RETURNING s.id, s.name, m.faculty_id, s.major_id, s.grade, s.term
        "#,
            subject_id.id(),
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("subject does not exist"))?;

        Ok(Subject::new(
            Id::new(deleted.id),
            deleted.name,
            Id::new(deleted.faculty_id),
            Id::new(deleted.major_id),
            Grade::new(deleted.grade)?,
            Term::new(deleted.term)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::{
        DocumentSeed, insert_document, insert_faculty_major, insert_subject, subject_of,
    };
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    // search_subjectsについて
    /// 条件を1つも指定しないとき全件返ることを確認
    #[sqlx::test]
    async fn search_subjects_returns_all_when_option_is_empty(pool: PgPool) {
        let (_, major_a) = insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let (_, major_b) = insert_faculty_major(&pool, "テスト学部B", "テスト専攻B").await;
        insert_subject(&pool, "テスト科目A", major_a, 1, 1).await;
        insert_subject(&pool, "テスト科目B", major_b, 2, 3).await;

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
        let (faculty_a_id, major_a) =
            insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let (_, major_b) = insert_faculty_major(&pool, "テスト学部B", "テスト専攻B").await;
        let subject_a = insert_subject(&pool, "テスト科目A", major_a, 1, 1).await;
        insert_subject(&pool, "テスト科目B", major_b, 2, 3).await;

        let repo = PostgresRepository::new(pool);
        let subjects = repo
            .search_subjects(SearchSubjectOption {
                faculty_id: Some(Id::new(faculty_a_id)),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].id().id(), &subject_a);
        assert_eq!(subjects[0].faculty_id().id(), &faculty_a_id);
    }

    /// 学年と学期という複数条件がANDで結合されることを確認
    #[sqlx::test]
    async fn search_subjects_filters_by_grade_and_term(pool: PgPool) {
        let (_, major_a) = insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let target = insert_subject(&pool, "テスト科目A", major_a, 2, 3).await;
        // 学年のみ一致
        insert_subject(&pool, "テスト科目B", major_a, 2, 1).await;
        // 学期のみ一致
        insert_subject(&pool, "テスト科目C", major_a, 1, 3).await;

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
        let (faculty_a_id, major_a) =
            insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let subject_id = Uuid::new_v4();

        let repo = PostgresRepository::new(pool);
        repo.create_subject(subject_of(
            subject_id,
            "テスト科目A",
            faculty_a_id,
            major_a,
            1,
            2,
        ))
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
        assert_eq!(subject.name(), "テスト科目A");
        assert_eq!(subject.faculty_id().id(), &faculty_a_id);
        assert_eq!(subject.major_id().id(), &major_a);
        assert_eq!(subject.grade().grade(), &1);
        assert_eq!(subject.term().term(), &2);
    }

    /// id重複がエラーとなり, 既存の行が書き換わらないことを確認
    #[sqlx::test]
    async fn create_subject_rejects_duplicate_id(pool: PgPool) {
        let (faculty_a_id, major_a) =
            insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let subject_id = Uuid::new_v4();

        let repo = PostgresRepository::new(pool);
        repo.create_subject(subject_of(
            subject_id,
            "テスト科目A",
            faculty_a_id,
            major_a,
            1,
            2,
        ))
        .await
        .unwrap();

        let result = repo
            .create_subject(subject_of(
                subject_id,
                "テスト科目B",
                faculty_a_id,
                major_a,
                3,
                4,
            ))
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
        assert_eq!(subjects[0].name(), "テスト科目A");
    }

    /// 専攻が学部に属さない組み合わせが弾かれることを確認
    #[sqlx::test]
    async fn create_subject_rejects_major_faculty_mismatch(pool: PgPool) {
        let (faculty_a_id, _) = insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let (_, major_b) = insert_faculty_major(&pool, "テスト学部B", "テスト専攻B").await;

        let repo = PostgresRepository::new(pool);
        // テスト学部Aにテスト専攻Bを組み合わせる
        let result = repo
            .create_subject(subject_of(
                Uuid::new_v4(),
                "テスト科目A",
                faculty_a_id,
                major_b,
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
        let (faculty_a_id, major_a) =
            insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let (faculty_b_id, major_b) =
            insert_faculty_major(&pool, "テスト学部B", "テスト専攻B").await;
        let subject_id = insert_subject(&pool, "テスト科目A", major_a, 1, 2).await;

        let repo = PostgresRepository::new(pool);
        let updated = repo
            .update_subject(
                Id::new(subject_id),
                UpdateSubjectContent {
                    name: "テスト科目B".to_owned(),
                    faculty_id: Id::new(faculty_b_id),
                    major_id: Id::new(major_b),
                    grade: Grade::new(3).unwrap(),
                    term: Term::new(4).unwrap(),
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.id().id(), &subject_id);
        assert_eq!(updated.name(), "テスト科目B");
        assert_eq!(updated.faculty_id().id(), &faculty_b_id);
        assert_eq!(updated.major_id().id(), &major_b);
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
        assert_eq!(stored[0].faculty_id().id(), &faculty_b_id);
        assert_ne!(stored[0].faculty_id().id(), &faculty_a_id);
    }

    /// 存在しないidの更新がエラーとなることを確認
    #[sqlx::test]
    async fn update_subject_errors_when_not_found(pool: PgPool) {
        let (faculty_a_id, major_a) =
            insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;

        let repo = PostgresRepository::new(pool);
        let result = repo
            .update_subject(
                Id::new(Uuid::new_v4()),
                UpdateSubjectContent {
                    name: "テスト科目A".to_owned(),
                    faculty_id: Id::new(faculty_a_id),
                    major_id: Id::new(major_a),
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
        let (faculty_a_id, major_a) =
            insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let (_, major_b) = insert_faculty_major(&pool, "テスト学部B", "テスト専攻B").await;
        let subject_id = insert_subject(&pool, "テスト科目A", major_a, 1, 2).await;

        let repo = PostgresRepository::new(pool);
        // テスト学部Aにテスト専攻Bを組み合わせる
        let result = repo
            .update_subject(
                Id::new(subject_id),
                UpdateSubjectContent {
                    name: "テスト科目B".to_owned(),
                    faculty_id: Id::new(faculty_a_id),
                    major_id: Id::new(major_b),
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
        assert_eq!(stored[0].name(), "テスト科目A");
    }

    // delete_subjectについて
    /// 削除前の値が返り, 行が消えることを確認
    #[sqlx::test]
    async fn delete_subject_returns_deleted_subject(pool: PgPool) {
        let (faculty_a_id, major_a) =
            insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let subject_id = insert_subject(&pool, "テスト科目A", major_a, 1, 2).await;

        let repo = PostgresRepository::new(pool);
        let deleted = repo.delete_subject(Id::new(subject_id)).await.unwrap();

        assert_eq!(deleted.id().id(), &subject_id);
        assert_eq!(deleted.name(), "テスト科目A");
        assert_eq!(deleted.faculty_id().id(), &faculty_a_id);
        assert_eq!(deleted.major_id().id(), &major_a);
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
        let (_, major_a) = insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let subject_id = insert_subject(&pool, "テスト科目A", major_a, 1, 2).await;
        insert_document(&pool, subject_id, DocumentSeed::default()).await;

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
}
