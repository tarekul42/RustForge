use sqlx::PgPool;
use sw_domain::aggregates::workshop::{Workshop, WorkshopImage};
use sw_domain::error::DomainError;
use sw_domain::repositories::workshop::WorkshopRepository;
use sw_domain::value_objects::ids::{CategoryId, LevelId, UserId, WorkshopId, WorkshopImageId};

/// SQLx-backed implementation of [`WorkshopRepository`].
pub struct PostgresWorkshopRepository {
    pool: PgPool,
}

impl PostgresWorkshopRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl WorkshopRepository for PostgresWorkshopRepository {
    async fn create(&self, workshop: &Workshop) -> Result<(), DomainError> {
        sqlx::query!(
            r#"INSERT INTO workshops (id, title, slug, description, location, price_cents,
               start_date, end_date, max_seats, current_enrollments, min_age,
               category_id, level_id, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)"#,
            workshop.id().into_uuid(),
            workshop.title(),
            workshop.slug(),
            workshop.description(),
            workshop.location(),
            workshop.price_cents(),
            workshop.start_date(),
            workshop.end_date(),
            workshop.max_seats(),
            workshop.current_enrollments(),
            workshop.min_age(),
            workshop.category_id().into_uuid(),
            workshop.level_id().into_uuid(),
            workshop.created_by().into_uuid(),
            workshop.created_at(),
            workshop.updated_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create workshop: {e}")))?;
        Ok(())
    }

    async fn find_by_id(&self, id: WorkshopId) -> Result<Option<Workshop>, DomainError> {
        let row = sqlx::query_as!(
            WorkshopRow,
            r#"SELECT id, title, slug, description, location, price_cents,
                      start_date, end_date, max_seats, current_enrollments, min_age,
                      category_id, level_id, created_by, created_at, updated_at
               FROM workshops WHERE id = $1"#,
            id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find workshop: {e}")))?;
        row.map(WorkshopRow::into_domain).transpose()
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Workshop>, DomainError> {
        let row = sqlx::query_as!(
            WorkshopRow,
            r#"SELECT id, title, slug, description, location, price_cents,
                      start_date, end_date, max_seats, current_enrollments, min_age,
                      category_id, level_id, created_by, created_at, updated_at
               FROM workshops WHERE slug = $1"#,
            slug,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::infrastructure(format!("failed to find workshop by slug: {e}"))
        })?;
        row.map(WorkshopRow::into_domain).transpose()
    }

    async fn update(&self, workshop: &Workshop) -> Result<(), DomainError> {
        sqlx::query!(
            r#"UPDATE workshops SET title = $2, slug = $3, description = $4, location = $5,
               price_cents = $6, start_date = $7, end_date = $8, max_seats = $9,
               current_enrollments = $10, min_age = $11, category_id = $12, level_id = $13,
               updated_at = $14
               WHERE id = $1"#,
            workshop.id().into_uuid(),
            workshop.title(),
            workshop.slug(),
            workshop.description(),
            workshop.location(),
            workshop.price_cents(),
            workshop.start_date(),
            workshop.end_date(),
            workshop.max_seats(),
            workshop.current_enrollments(),
            workshop.min_age(),
            workshop.category_id().into_uuid(),
            workshop.level_id().into_uuid(),
            workshop.updated_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to update workshop: {e}")))?;
        Ok(())
    }

    async fn delete(&self, id: WorkshopId) -> Result<(), DomainError> {
        sqlx::query!("DELETE FROM workshops WHERE id = $1", id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to delete workshop: {e}")))?;
        Ok(())
    }

    async fn get_images(&self, workshop_id: WorkshopId) -> Result<Vec<WorkshopImage>, DomainError> {
        let rows = sqlx::query_as!(
            WorkshopImageRow,
            r#"SELECT id, workshop_id, url, s3_key, created_at
               FROM workshop_images WHERE workshop_id = $1 ORDER BY created_at"#,
            workshop_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to get workshop images: {e}")))?;
        rows.into_iter()
            .map(WorkshopImageRow::into_domain)
            .collect()
    }

    async fn add_image(
        &self,
        workshop_id: WorkshopId,
        url: &str,
        s3_key: &str,
    ) -> Result<WorkshopImage, DomainError> {
        let id = uuid::Uuid::now_v7();
        let now = chrono::Utc::now();
        sqlx::query!(
            r#"INSERT INTO workshop_images (id, workshop_id, url, s3_key, created_at)
               VALUES ($1, $2, $3, $4, $5)"#,
            id,
            workshop_id.into_uuid(),
            url,
            s3_key,
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to add workshop image: {e}")))?;
        Ok(WorkshopImage::from_parts(
            WorkshopImageId::from_uuid(id),
            workshop_id,
            url.to_string(),
            s3_key.to_string(),
            now,
        ))
    }

    async fn remove_image(&self, image_id: WorkshopImageId) -> Result<(), DomainError> {
        sqlx::query!("DELETE FROM workshop_images WHERE id = $1", image_id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DomainError::infrastructure(format!("failed to remove workshop image: {e}"))
            })?;
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Workshop>, DomainError> {
        let rows = sqlx::query_as!(
            WorkshopRow,
            r#"SELECT id, title, slug, description, location, price_cents,
                      start_date, end_date, max_seats, current_enrollments, min_age,
                      category_id, level_id, created_by, created_at, updated_at
               FROM workshops ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to list workshops: {e}")))?;
        rows.into_iter().map(WorkshopRow::into_domain).collect()
    }

    async fn reserve_seat_atomic(
        &self,
        workshop_id: WorkshopId,
    ) -> Result<Option<Workshop>, DomainError> {
        let row = sqlx::query_as!(
            WorkshopRow,
            r#"UPDATE workshops
               SET current_enrollments = current_enrollments + 1, updated_at = NOW()
               WHERE id = $1 AND (max_seats IS NULL OR current_enrollments < max_seats)
               RETURNING id, title, slug, description, location, price_cents,
                         start_date, end_date, max_seats, current_enrollments, min_age,
                         category_id, level_id, created_by, created_at, updated_at"#,
            workshop_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to reserve seat: {e}")))?;
        row.map(WorkshopRow::into_domain).transpose()
    }

    async fn release_seat_atomic(&self, workshop_id: WorkshopId) -> Result<(), DomainError> {
        sqlx::query!(
            r#"UPDATE workshops
               SET current_enrollments = GREATEST(current_enrollments - 1, 0), updated_at = NOW()
               WHERE id = $1 AND current_enrollments > 0"#,
            workshop_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to release seat: {e}")))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct WorkshopRow {
    id: uuid::Uuid,
    title: String,
    slug: String,
    description: Option<String>,
    location: Option<String>,
    price_cents: i64,
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    end_date: Option<chrono::DateTime<chrono::Utc>>,
    max_seats: Option<i32>,
    current_enrollments: i32,
    min_age: Option<i16>,
    category_id: uuid::Uuid,
    level_id: uuid::Uuid,
    created_by: uuid::Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl WorkshopRow {
    fn into_domain(self) -> Result<Workshop, DomainError> {
        Ok(Workshop::from_parts(
            WorkshopId::from_uuid(self.id),
            self.title,
            self.slug,
            self.description,
            self.location,
            self.price_cents,
            self.start_date,
            self.end_date,
            self.max_seats,
            self.current_enrollments,
            self.min_age,
            CategoryId::from_uuid(self.category_id),
            LevelId::from_uuid(self.level_id),
            UserId::from_uuid(self.created_by),
            self.created_at,
            self.updated_at,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct WorkshopImageRow {
    id: uuid::Uuid,
    workshop_id: uuid::Uuid,
    url: String,
    s3_key: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl WorkshopImageRow {
    fn into_domain(self) -> Result<WorkshopImage, DomainError> {
        Ok(WorkshopImage::from_parts(
            WorkshopImageId::from_uuid(self.id),
            WorkshopId::from_uuid(self.workshop_id),
            self.url,
            self.s3_key,
            self.created_at,
        ))
    }
}
