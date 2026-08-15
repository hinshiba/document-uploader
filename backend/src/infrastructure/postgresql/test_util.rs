//! テスト用のヘルパー群
//!
//! 学部・専攻はマイグレーションのシードと衝突しないよう, 
//! テスト内では`テスト学部A`のような明らかにテスト用と分かる名前を用いる

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Grade, Id, Term, document::ExamType, subject::Subject};

/// 指定した学部と学科群のペアを挿入する
pub(super) async fn insert_faculty_majors(
    pool: &PgPool,
    faculty: &str,
    majors: Vec<&str>,
) -> (Uuid, Vec<Uuid>) {
    let faculty_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO faculties (id, name)
            VALUES ($1, $2)",
        faculty_id,
        faculty
    )
    .execute(pool)
    .await
    .unwrap();

    let mut major_ids = vec![];
    for major in majors {
        let major_id = Uuid::new_v4();
        major_ids.push(major_id);
        sqlx::query!(
            "INSERT INTO majors (id, name, faculty_id)
                VALUES ($1, $2, $3)",
            major_id,
            major,
            faculty_id
        )
        .execute(pool)
        .await
        .unwrap();
    }
    (faculty_id, major_ids)
}

/// 専攻を1つだけ持つ学部を挿入する
///
/// 戻り値は`(学部id, 専攻id)`
pub(super) async fn insert_faculty_major(
    pool: &PgPool,
    faculty: &str,
    major: &str,
) -> (Uuid, Uuid) {
    let (faculty_id, major_ids) = insert_faculty_majors(pool, faculty, vec![major]).await;
    (faculty_id, major_ids[0])
}

pub(super) fn subject_of(
    id: Uuid,
    name: &str,
    faculty_id: Uuid,
    major_id: Uuid,
    grade: i64,
    term: i64,
) -> Subject {
    Subject::new(
        Id::new(id),
        name.to_owned(),
        Id::new(faculty_id),
        Id::new(major_id),
        Grade::new(grade).unwrap(),
        Term::new(term).unwrap(),
    )
}

/// `subjects`へ直接投入し, そのidを返す
///
/// faculty_idはmajor_id経由でしか解決されない
pub(super) async fn insert_subject(
    pool: &PgPool,
    name: &str,
    major_id: Uuid,
    grade: i64,
    term: i64,
) -> Uuid {
    let subject_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO subjects (id, name, major_id, grade, term)
            VALUES ($1, $2, $3, $4, $5)",
        subject_id,
        name,
        major_id,
        grade,
        term
    )
    .execute(pool)
    .await
    .unwrap();
    subject_id
}

/// `documents`へ投入する値
///
/// 着目しない列は`..Default::default()`に任せ, テストの意図を差分で示す
#[derive(Clone, Copy)]
pub(super) struct DocumentSeed<'a> {
    pub year: i64,
    pub teacher: &'a str,
    pub exam_type: ExamType,
    pub is_answer: bool,
    pub num: i64,
}

impl Default for DocumentSeed<'_> {
    fn default() -> Self {
        Self {
            year: 2024,
            teacher: "テスト教員A",
            exam_type: ExamType::FinalTerm,
            is_answer: false,
            num: 1,
        }
    }
}

/// `documents`へ直接投入し, そのidを返す
pub(super) async fn insert_document(
    pool: &PgPool,
    subject_id: Uuid,
    seed: DocumentSeed<'_>,
) -> Uuid {
    let document_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO documents (id, subject_id, year, teacher, exam_type, is_answer, num)
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
        document_id,
        subject_id,
        seed.year,
        seed.teacher,
        seed.exam_type.to_int(),
        seed.is_answer,
        seed.num
    )
    .execute(pool)
    .await
    .unwrap();
    document_id
}

/// `document_files`へ拡張子をfile_typeとして投入する
pub(super) async fn insert_document_files(pool: &PgPool, document_id: Uuid, paths: &[&str]) {
    for path in paths {
        let file_type = path.rsplit('.').next().expect("拡張子なし");
        sqlx::query!(
            "INSERT INTO document_files (document_id, file_type, path)
                VALUES ($1, $2, $3)",
            document_id,
            file_type,
            path
        )
        .execute(pool)
        .await
        .unwrap();
    }
}
