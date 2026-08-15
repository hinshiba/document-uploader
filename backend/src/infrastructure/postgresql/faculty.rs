use std::collections::HashMap;

use crate::{
    domain::{Id, faculty::Faculty, major::Major},
    usecase::repository::FacultyRepository,
};

use super::PostgresRepository;

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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::infrastructure::postgresql::test_util::insert_faculty_majors;
    use sqlx::PgPool;

    // list_facultiesについて
    /// 専攻が0の要素の列挙可能性を確認
    #[sqlx::test]
    async fn list_faculties_zero_majors(pool: PgPool) {
        // 初期値の生成
        let (fid, _mids) = insert_faculty_majors(&pool, "専攻ゼロ学部", vec![]).await;

        // 実行
        let repo = PostgresRepository::new(pool);
        let faculties = repo.list_faculties().await.unwrap();

        // 検査
        let faculty = faculties.iter().find(|f| f.id().id() == &fid).unwrap();
        assert_eq!(faculty.name(), "専攻ゼロ学部");
        assert!(faculty.majors().is_empty());
    }

    /// 学部と専攻が適切な組になっているか確認
    #[sqlx::test]
    async fn list_faculties_pairs_majors_with_own_faculty(pool: PgPool) {
        // 初期値の生成
        let (faculty_a_id, faculty_a_major_ids) =
            insert_faculty_majors(&pool, "テスト学部A", vec!["テスト専攻A1", "テスト専攻A2"]).await;
        let (faculty_b_id, faculty_b_major_ids) =
            insert_faculty_majors(&pool, "テスト学部B", vec!["テスト専攻B1"]).await;

        // 実行
        let repo = PostgresRepository::new(pool);
        let faculties = repo.list_faculties().await.unwrap();

        // 検査
        let faculty_a = faculties
            .iter()
            .find(|f| f.id().id() == &faculty_a_id)
            .unwrap();
        assert_eq!(faculty_a.name(), "テスト学部A");
        let faculty_a_majors: HashSet<_> = faculty_a.majors().iter().map(|m| *m.id().id()).collect();
        assert_eq!(faculty_a_majors, faculty_a_major_ids.into_iter().collect());
        for m in faculty_a.majors() {
            assert_eq!(m.faculty_id().id(), &faculty_a_id);
        }

        let faculty_b = faculties
            .iter()
            .find(|f| f.id().id() == &faculty_b_id)
            .unwrap();
        assert_eq!(faculty_b.name(), "テスト学部B");
        let faculty_b_majors: HashSet<_> = faculty_b.majors().iter().map(|m| *m.id().id()).collect();
        assert_eq!(faculty_b_majors, faculty_b_major_ids.into_iter().collect());
        for m in faculty_b.majors() {
            assert_eq!(m.faculty_id().id(), &faculty_b_id);
        }
    }
}
