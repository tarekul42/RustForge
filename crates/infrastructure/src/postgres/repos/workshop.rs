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
        sqlx::query(
            r#"INSERT INTO workshops (id, title, slug, description, location, price_cents,
               start_date, end_date, max_seats, current_enrollments, min_age,
               category_id, level_id, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)"#,
        )
        .bind(workshop.id.into_uuid())
        .bind(&workshop.title)
        .bind(&workshop.slug)
        .bind(&workshop.description)
        .bind(&workshop.location)
        .bind(workshop.price_cents)
        .bind(workshop.start_date)
        .bind(workshop.end_date)
        .bind(workshop.max_seats)
        .bind(workshop.current_enrollments)
        .bind(workshop.min_age)
        .bind(workshop.category_id.into_uuid())
        .bind(workshop.level_id.into_uuid())
        .bind(workshop.created_by.into_uuid())
        .bind(workshop.created_at)
        .bind(workshop.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create workshop: {e}")))?;
        Ok(())
    }

    async fn find_by_id(&self, id: WorkshopId) -> Result<Option<Workshop>, DomainError> {
        let row = sqlx::query_as::<_, WorkshopRow>(
            r#"SELECT id, title, slug, description, location, price_cents,
                      start_date, end_date, max_seats, current_enrollments, min_age,
                      category_id, level_id, created_by, created_at, updated_at
               FROM workshops WHERE id = $1"#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find workshop: {e}")))?;
        row.map(WorkshopRow::into_domain).transpose()
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Workshop>, DomainError> {
        let row = sqlx::query_as::<_, WorkshopRow>(
            r#"SELECT id, title, slug, description, location, price_cents,
                      start_date, end_date, max_seats, current_enrollments, min_age,
                      category_id, level_id, created_by, created_at, updated_at
               FROM workshops WHERE slug = $1"#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::infrastructure(format!("failed to find workshop by slug: {e}"))
        })?;
        row.map(WorkshopRow::into_domain).transpose()
    }

    async fn update(&self, workshop: &Workshop) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE workshops SET title = $2, slug = $3, description = $4, location = $5,
               price_cents = $6, start_date = $7, end_date = $8, max_seats = $9,
               current_enrollments = $10, min_age = $11, category_id = $12, level_id = $13,
               updated_at = $14
               WHERE id = $1"#,
        )
        .bind(workshop.id.into_uuid())
        .bind(&workshop.title)
        .bind(&workshop.slug)
        .bind(&workshop.description)
        .bind(&workshop.location)
        .bind(workshop.price_cents)
        .bind(workshop.start_date)
        .bind(workshop.end_date)
        .bind(workshop.max_seats)
        .bind(workshop.current_enrollments)
        .bind(workshop.min_age)
        .bind(workshop.category_id.into_uuid())
        .bind(workshop.level_id.into_uuid())
        .bind(workshop.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to update workshop: {e}")))?;
        Ok(())
    }

    async fn delete(&self, id: WorkshopId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM workshops WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to delete workshop: {e}")))?;
        Ok(())
    }

    async fn get_images(&self, workshop_id: WorkshopId) -> Result<Vec<WorkshopImage>, DomainError> {
        let rows = sqlx::query_as::<_, WorkshopImageRow>(
            r#"SELECT id, workshop_id, url, s3_key, created_at
               FROM workshop_images WHERE workshop_id = $1 ORDER BY created_at"#,
        )
        .bind(workshop_id.into_uuid())
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
        sqlx::query(
            r#"INSERT INTO workshop_images (id, workshop_id, url, s3_key, created_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(workshop_id.into_uuid())
        .bind(url)
        .bind(s3_key)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to add workshop image: {e}")))?;
        Ok(WorkshopImage {
            id: WorkshopImageId::from_uuid(id),
            workshop_id,
            url: url.to_string(),
            s3_key: s3_key.to_string(),
            created_at: now,
        })
    }

    async fn remove_image(&self, image_id: WorkshopImageId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM workshop_images WHERE id = $1")
            .bind(image_id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DomainError::infrastructure(format!("failed to remove workshop image: {e}"))
            })?;
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
        Ok(Workshop {
            id: WorkshopId::from_uuid(self.id),
            title: self.title,
            slug: self.slug,
            description: self.description,
            location: self.location,
            price_cents: self.price_cents,
            start_date: self.start_date,
            end_date: self.end_date,
            max_seats: self.max_seats,
            current_enrollments: self.current_enrollments,
            min_age: self.min_age,
            category_id: CategoryId::from_uuid(self.category_id),
            level_id: LevelId::from_uuid(self.level_id),
            created_by: UserId::from_uuid(self.created_by),
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
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
        Ok(WorkshopImage {
            id: WorkshopImageId::from_uuid(self.id),
            workshop_id: WorkshopId::from_uuid(self.workshop_id),
            url: self.url,
            s3_key: self.s3_key,
            created_at: self.created_at,
        })
    }
}
