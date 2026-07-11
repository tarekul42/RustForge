use sqlx::PgPool;
use sw_domain::aggregates::level::Level;
use sw_domain::error::DomainError;
use sw_domain::repositories::level::LevelRepository;
use sw_domain::value_objects::ids::LevelId;

pub struct PostgresLevelRepository {
    pool: PgPool,
}

impl PostgresLevelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl LevelRepository for PostgresLevelRepository {
    async fn create(&self, level: &Level) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO levels (id, name, created_at, updated_at)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(level.id.into_uuid())
        .bind(&level.name)
        .bind(level.created_at)
        .bind(level.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create level: {e}")))?;
        Ok(())
    }

    async fn find_by_id(&self, id: LevelId) -> Result<Option<Level>, DomainError> {
        let row = sqlx::query_as::<_, LevelRow>(
            r#"SELECT id, name, created_at, updated_at
               FROM levels WHERE id = $1"#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find level: {e}")))?;
        row.map(LevelRow::into_domain).transpose()
    }

    async fn find_all(&self) -> Result<Vec<Level>, DomainError> {
        let rows = sqlx::query_as::<_, LevelRow>(
            r#"SELECT id, name, created_at, updated_at
               FROM levels ORDER BY name"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to list levels: {e}")))?;
        rows.into_iter().map(LevelRow::into_domain).collect()
    }

    async fn update(&self, level: &Level) -> Result<(), DomainError> {
        sqlx::query(r#"UPDATE levels SET name = $2, updated_at = $3 WHERE id = $1"#)
            .bind(level.id.into_uuid())
            .bind(&level.name)
            .bind(level.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to update level: {e}")))?;
        Ok(())
    }

    async fn delete(&self, id: LevelId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM levels WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to delete level: {e}")))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct LevelRow {
    id: uuid::Uuid,
    name: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl LevelRow {
    fn into_domain(self) -> Result<Level, DomainError> {
        Ok(Level {
            id: LevelId::from_uuid(self.id),
            name: self.name,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
