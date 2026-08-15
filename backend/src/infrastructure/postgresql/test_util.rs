//! テスト用のヘルパー群

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Grade, Id, Term, subject::Subject};

/// 指定した学部と学科群のペアを挿入する
pub(super) async fn insert_faculty_majors(pool: &PgPool, faculty: &str, majors: Vec<&str>) -> (Uuid, Vec<Uuid>) {
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

/// `subjects`へ直接投入する, faculty_idはmajor_id経由でしか解決されない
pub(super) async fn insert_subject(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    major_id: Uuid,
    grade: i64,
    term: i64,
) {
    sqlx::query!(
        "INSERT INTO subjects (id, name, major_id, grade, term)
            VALUES ($1, $2, $3, $4, $5)",
        id,
        name,
        major_id,
        grade,
        term
    )
    .execute(pool)
    .await
    .unwrap();
}

/// `documents`へ直接投入する
#[allow(clippy::too_many_arguments)]
pub(super) async fn insert_document(
    pool: &PgPool,
    id: Uuid,
    subject_id: Uuid,
    year: i64,
    teacher: &str,
    exam_type: i64,
    is_answer: bool,
    num: i64,
) {
    sqlx::query!(
        "INSERT INTO documents (id, subject_id, year, teacher, exam_type, is_answer, num)
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
        id,
        subject_id,
        year,
        teacher,
        exam_type,
        is_answer,
        num
    )
    .execute(pool)
    .await
    .unwrap();
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
