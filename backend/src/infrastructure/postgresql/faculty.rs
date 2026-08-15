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
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

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
}
