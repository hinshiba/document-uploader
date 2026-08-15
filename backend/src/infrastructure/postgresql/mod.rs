mod document;
mod faculty;
mod subject;

#[cfg(test)]
mod test_util;

use sqlx::PgPool;

#[derive(Debug)]
pub struct PostgresRepository {
    pool: sqlx::PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 空のDBがあるか確認
    #[sqlx::test]
    async fn migrations_run(pool: PgPool) {
        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM faculties")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, Some(0));
    }
}
