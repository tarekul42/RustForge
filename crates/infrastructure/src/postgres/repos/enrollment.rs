use sqlx::PgPool;
use sw_domain::aggregates::enrollment::{Enrollment, EnrollmentStatus};
use sw_domain::error::DomainError;
use sw_domain::repositories::enrollment::EnrollmentRepository;
use sw_domain::value_objects::ids::{EnrollmentId, PaymentId, UserId, WorkshopId};

/// SQLx-backed implementation of [`EnrollmentRepository`].
pub struct PostgresEnrollmentRepository {
    pool: PgPool,
}

impl PostgresEnrollmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EnrollmentRepository for PostgresEnrollmentRepository {
    async fn create(&self, enrollment: &Enrollment) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO enrollments (id, user_id, workshop_id, payment_id, student_count, status, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(enrollment.id.into_uuid())
        .bind(enrollment.user_id.into_uuid())
        .bind(enrollment.workshop_id.into_uuid())
        .bind(enrollment.payment_id.map(|p| p.into_uuid()))
        .bind(enrollment.student_count)
        .bind(enrollment.status.as_str())
        .bind(enrollment.created_at)
        .bind(enrollment.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create enrollment: {e}")))?;
        Ok(())
    }

    async fn find_by_id(&self, id: EnrollmentId) -> Result<Option<Enrollment>, DomainError> {
        let row = sqlx::query_as::<_, EnrollmentRow>(
            r#"SELECT id, user_id, workshop_id, payment_id, student_count, status, created_at, updated_at
               FROM enrollments WHERE id = $1"#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find enrollment: {e}")))?;
        row.map(EnrollmentRow::into_domain).transpose()
    }

    async fn find_by_user_and_workshop(
        &self,
        user_id: UserId,
        workshop_id: WorkshopId,
    ) -> Result<Vec<Enrollment>, DomainError> {
        let rows = sqlx::query_as::<_, EnrollmentRow>(
            r#"SELECT id, user_id, workshop_id, payment_id, student_count, status, created_at, updated_at
               FROM enrollments
               WHERE user_id = $1 AND workshop_id = $2
               ORDER BY created_at DESC"#,
        )
        .bind(user_id.into_uuid())
        .bind(workshop_id.into_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find enrollments: {e}")))?;
        rows.into_iter().map(EnrollmentRow::into_domain).collect()
    }

    async fn update(&self, enrollment: &Enrollment) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE enrollments SET payment_id = $2, student_count = $3, status = $4, updated_at = $5
               WHERE id = $1"#,
        )
        .bind(enrollment.id.into_uuid())
        .bind(enrollment.payment_id.map(|p| p.into_uuid()))
        .bind(enrollment.student_count)
        .bind(enrollment.status.as_str())
        .bind(enrollment.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to update enrollment: {e}")))?;
        Ok(())
    }

    async fn delete(&self, id: EnrollmentId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM enrollments WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DomainError::infrastructure(format!("failed to delete enrollment: {e}"))
            })?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct EnrollmentRow {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    workshop_id: uuid::Uuid,
    payment_id: Option<uuid::Uuid>,
    student_count: i32,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl EnrollmentRow {
    fn into_domain(self) -> Result<Enrollment, DomainError> {
        let status = EnrollmentStatus::from_str(&self.status).ok_or_else(|| {
            DomainError::infrastructure(format!("invalid enrollment status: {}", self.status))
        })?;
        Ok(Enrollment {
            id: EnrollmentId::from_uuid(self.id),
            user_id: UserId::from_uuid(self.user_id),
            workshop_id: WorkshopId::from_uuid(self.workshop_id),
            payment_id: self.payment_id.map(PaymentId::from_uuid),
            student_count: self.student_count,
            status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
