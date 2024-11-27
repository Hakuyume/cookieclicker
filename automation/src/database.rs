use std::fmt;
use std::path::Path;

pub struct Database(sqlx::SqlitePool);

impl Database {
    #[tracing::instrument(err)]
    pub async fn connect<P>(filename: P) -> anyhow::Result<Self>
    where
        P: fmt::Debug + AsRef<Path>,
    {
        let pool = sqlx::SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .create_if_missing(true)
                .filename(filename),
        )
        .await?;
        let mut tx = pool.begin().await?;
        sqlx::query(concat!(
            "CREATE TABLE IF NOT EXISTS saves(",
            "id INTEGER PRIMARY KEY AUTOINCREMENT,",
            "timestamp TEXT,",
            "value TEXT,",
            "cookies_baked_all_time REAL)"
        ))
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS cookies_baked_all_time ON saves(cookies_baked_all_time)",
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(Self(pool))
    }

    #[tracing::instrument(err, ret, skip(self))]
    pub async fn fetch_save_best(&self) -> anyhow::Result<Option<crate::save::Save>> {
        if let Some((timestamp, code)) = sqlx::query_as(
            "SELECT timestamp, value FROM saves ORDER BY cookies_baked_all_time DESC LIMIT 1",
        )
        .fetch_optional(&self.0)
        .await?
        {
            Ok(Some(crate::save::Save::new(timestamp, code)?))
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(err, ret, skip(self))]
    pub async fn insert_save(&self, save: &crate::save::Save) -> anyhow::Result<i64> {
        let id = sqlx::query(concat!(
            "INSERT OR REPLACE INTO saves",
            "(timestamp, value, cookies_baked_all_time)",
            "values (?, ?, ?)",
        ))
        .bind(save.timestamp())
        .bind(save.code())
        .bind(save.cookies_baked_all_time())
        .execute(&self.0)
        .await?
        .last_insert_rowid();
        Ok(id)
    }
}
