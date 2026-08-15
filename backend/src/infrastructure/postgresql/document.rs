use std::collections::HashMap;

use crate::{
    domain::{
        Grade, Id, Num, Term, Year,
        document::{Document, DocumentFile, DocumentMetadata, ExamType},
    },
    usecase::repository::{DocumentRepository, SearchDocumentOption},
};

use super::PostgresRepository;

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

    #[tracing::instrument(skip(self), err(Display))]
    async fn search_documents(
        &self,
        option: SearchDocumentOption,
    ) -> anyhow::Result<Vec<Document>> {
        // subject_idは必須, それ以外はNULLなら絞り込みに寄与させない
        let rows = sqlx::query!(
            r#"
            SELECT
                d.id, d.year, d.teacher, d.exam_type, d.is_answer, d.num,
                s.id AS "subject_id!", s.faculty_id AS "faculty_id!",
                s.major_id AS "major_id!", s.grade AS "grade!", s.term AS "term!"
            FROM documents AS d
                INNER JOIN subject_details AS s ON s.id = d.subject_id
            WHERE
                d.subject_id = $1 AND
                ($2::bigint IS NULL OR d.year = $2) AND
                ($3::text IS NULL OR d.teacher = $3) AND
                ($4::bigint IS NULL OR d.exam_type = $4) AND
                ($5::boolean IS NULL OR d.is_answer = $5)
        "#,
            option.subject_id.id(),
            option.year.as_ref().map(|year| *year.year()),
            option.teacher.as_deref(),
            option.exam_type.map(|exam_type| exam_type.to_int()),
            option.is_answer,
        )
        .fetch_all(&self.pool)
        .await?;

        // 紐づくファイル情報をまとめて取得する
        let document_ids: Vec<uuid::Uuid> = rows.iter().map(|r| r.id).collect();
        let mut file_map: HashMap<uuid::Uuid, Vec<DocumentFile>> = HashMap::new();
        for f in sqlx::query!(
            r#"
            SELECT document_id, file_type, path
            FROM document_files
            WHERE document_id = ANY($1)
        "#,
            &document_ids,
        )
        .fetch_all(&self.pool)
        .await?
        {
            file_map
                .entry(f.document_id)
                .or_default()
                .push(DocumentFile::new(f.file_type.parse()?, f.path.into()));
        }

        // mapをremoveしながら生成
        rows.into_iter()
            .map(|r| {
                let metadata = DocumentMetadata::new(
                    Id::new(r.faculty_id),
                    Id::new(r.major_id),
                    Year::new(r.year)?,
                    Term::new(r.term)?,
                    Grade::new(r.grade)?,
                    Id::new(r.subject_id),
                    r.teacher,
                    ExamType::from_int(r.exam_type)
                        .ok_or_else(|| anyhow::anyhow!("Invalid exam_type stored in database."))?,
                    r.is_answer,
                    Num::new(r.num)?,
                );

                Ok(Document::new(
                    Id::new(r.id),
                    metadata,
                    file_map.remove(&r.id).unwrap_or_default(),
                )?)
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::{
        DocumentSeed, insert_document, insert_document_files, insert_faculty_major, insert_subject,
    };
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    // find_document_by_idについて
    /// subjects,majorsをjoinしてメタデータ・ファイルを復元できるか確認
    #[sqlx::test]
    async fn find_document_by_id_reconstructs_document(pool: PgPool) {
        // 初期値の生成
        let (faculty_id, major_id) =
            insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let subject_id = insert_subject(&pool, "テスト科目A", major_id, 1, 2).await;
        let document_id = insert_document(
            &pool,
            subject_id,
            DocumentSeed {
                year: 2024,
                teacher: "テスト教員A",
                exam_type: ExamType::FinalTerm,
                is_answer: false,
                num: 1,
            },
        )
        .await;
        insert_document_files(&pool, document_id, &["path/to/a.pdf", "path/to/b.jpg"]).await;

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
        assert_eq!(meta.teacher(), "テスト教員A");
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

    // search_documentsについて
    /// subject_idで絞り込まれ, ファイルが資料ごとに紐づくことを確認
    #[sqlx::test]
    async fn search_documents_filters_by_subject_id(pool: PgPool) {
        let (_, major_a) = insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let target_subject = insert_subject(&pool, "テスト科目A", major_a, 1, 2).await;
        let other_subject = insert_subject(&pool, "テスト科目B", major_a, 1, 2).await;

        let target = insert_document(&pool, target_subject, DocumentSeed::default()).await;
        insert_document_files(&pool, target, &["path/to/a.pdf", "path/to/b.jpg"]).await;
        let other = insert_document(&pool, other_subject, DocumentSeed::default()).await;
        insert_document_files(&pool, other, &["path/to/c.pdf"]).await;

        let repo = PostgresRepository::new(pool);
        let documents = repo
            .search_documents(SearchDocumentOption::minimal(Id::new(target_subject)))
            .await
            .unwrap();

        assert_eq!(documents.len(), 1);
        let document = &documents[0];
        assert_eq!(document.id().id(), &target);

        let meta = document.metadata();
        assert_eq!(meta.subject_id().id(), &target_subject);
        assert_eq!(meta.major_id().id(), &major_a);
        assert_eq!(meta.grade().grade(), &1);
        assert_eq!(meta.term().term(), &2);
        assert_eq!(meta.year().year(), &2024);
        assert_eq!(meta.exam_type(), &ExamType::FinalTerm);

        // 他の資料のファイルが混入しないこと
        let mut paths: Vec<_> = document
            .files()
            .iter()
            .map(|f| f.path().to_str().unwrap())
            .collect();
        paths.sort();
        assert_eq!(paths, ["path/to/a.pdf", "path/to/b.jpg"]);
    }

    /// 任意条件がANDで結合されることを確認
    #[sqlx::test]
    async fn search_documents_filters_by_optional_conditions(pool: PgPool) {
        let (_, major_a) = insert_faculty_major(&pool, "テスト学部A", "テスト専攻A").await;
        let subject_id = insert_subject(&pool, "テスト科目A", major_a, 1, 2).await;

        // 検索条件に一致する資料, 以降はここから1条件だけずらす
        let matched = DocumentSeed {
            year: 2024,
            teacher: "テスト教員A",
            exam_type: ExamType::FinalTerm,
            is_answer: true,
            num: 1,
        };
        let target = insert_document(&pool, subject_id, matched).await;
        insert_document_files(&pool, target, &["path/to/a.pdf"]).await;
        // 年度のみ不一致
        let other_year = insert_document(
            &pool,
            subject_id,
            DocumentSeed {
                year: 2023,
                ..matched
            },
        )
        .await;
        insert_document_files(&pool, other_year, &["path/to/b.pdf"]).await;
        // 教員のみ不一致
        let other_teacher = insert_document(
            &pool,
            subject_id,
            DocumentSeed {
                teacher: "テスト教員B",
                ..matched
            },
        )
        .await;
        insert_document_files(&pool, other_teacher, &["path/to/c.pdf"]).await;
        // 試験種別のみ不一致
        let other_exam_type = insert_document(
            &pool,
            subject_id,
            DocumentSeed {
                exam_type: ExamType::MidTerm,
                ..matched
            },
        )
        .await;
        insert_document_files(&pool, other_exam_type, &["path/to/d.pdf"]).await;
        // 解答か否かのみ不一致
        let other_is_answer = insert_document(
            &pool,
            subject_id,
            DocumentSeed {
                is_answer: false,
                ..matched
            },
        )
        .await;
        insert_document_files(&pool, other_is_answer, &["path/to/e.pdf"]).await;

        let repo = PostgresRepository::new(pool);
        let documents = repo
            .search_documents(SearchDocumentOption {
                year: Some(Year::new(matched.year).unwrap()),
                teacher: Some(matched.teacher.to_owned()),
                exam_type: Some(matched.exam_type),
                is_answer: Some(matched.is_answer),
                ..SearchDocumentOption::minimal(Id::new(subject_id))
            })
            .await
            .unwrap();

        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].id().id(), &target);
    }

    /// 該当なしのとき空のVecが返ることを確認
    #[sqlx::test]
    async fn search_documents_returns_empty_when_no_match(pool: PgPool) {
        let repo = PostgresRepository::new(pool);
        let documents = repo
            .search_documents(SearchDocumentOption::minimal(Id::new(Uuid::new_v4())))
            .await
            .unwrap();

        assert!(documents.is_empty());
    }
}
