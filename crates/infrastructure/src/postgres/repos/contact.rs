use sqlx::PgPool;
use sw_domain::aggregates::contact::Contact;
use sw_domain::error::DomainError;
use sw_domain::repositories::contact::ContactRepository;
use sw_domain::value_objects::Email;
use sw_domain::value_objects::ids::ContactId;

/// SQLx-backed implementation of [`ContactRepository`].
pub struct PostgresContactRepository {
    pool: PgPool,
}

impl PostgresContactRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ContactRepository for PostgresContactRepository {
    async fn create(&self, contact: &Contact) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO contacts (id, name, email, subject, message, is_read, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(contact.id().into_uuid())
        .bind(contact.name())
        .bind(contact.email().as_str())
        .bind(contact.subject())
        .bind(contact.message())
        .bind(contact.is_read())
        .bind(contact.created_at())
        .bind(contact.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create contact: {e}")))?;
        Ok(())
    }

    async fn find_by_id(&self, id: ContactId) -> Result<Option<Contact>, DomainError> {
        let row = sqlx::query_as::<_, ContactRow>(
            r#"SELECT id, name, email, subject, message, is_read, created_at, updated_at
               FROM contacts WHERE id = $1"#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find contact: {e}")))?;
        row.map(ContactRow::into_domain).transpose()
    }

    async fn list(&self, is_read: Option<bool>) -> Result<Vec<Contact>, DomainError> {
        let rows = match is_read {
            Some(read) => {
                sqlx::query_as::<_, ContactRow>(
                    r#"SELECT id, name, email, subject, message, is_read, created_at, updated_at
                       FROM contacts WHERE is_read = $1
                       ORDER BY created_at DESC"#,
                )
                .bind(read)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, ContactRow>(
                    r#"SELECT id, name, email, subject, message, is_read, created_at, updated_at
                       FROM contacts
                       ORDER BY created_at DESC"#,
                )
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| DomainError::infrastructure(format!("failed to list contacts: {e}")))?;
        rows.into_iter().map(ContactRow::into_domain).collect()
    }

    async fn update(&self, contact: &Contact) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE contacts SET name = $2, email = $3, subject = $4, message = $5, is_read = $6, updated_at = $7
               WHERE id = $1"#,
        )
        .bind(contact.id().into_uuid())
        .bind(contact.name())
        .bind(contact.email().as_str())
        .bind(contact.subject())
        .bind(contact.message())
        .bind(contact.is_read())
        .bind(contact.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to update contact: {e}")))?;
        Ok(())
    }

    async fn delete(&self, id: ContactId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM contacts WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to delete contact: {e}")))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ContactRow {
    id: uuid::Uuid,
    name: String,
    email: String,
    subject: String,
    message: String,
    is_read: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl ContactRow {
    fn into_domain(self) -> Result<Contact, DomainError> {
        let email = Email::new(self.email).map_err(|e| {
            DomainError::infrastructure(format!("invalid email in contact row: {e}"))
        })?;
        Ok(Contact::from_parts(
            ContactId::from_uuid(self.id),
            self.name,
            email,
            self.subject,
            self.message,
            self.is_read,
            self.created_at,
            self.updated_at,
        ))
    }
}
