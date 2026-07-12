use sqlx::PgPool;
use sw_domain::aggregates::user::{User, UserRole, UserStatus};
use sw_domain::error::DomainError;
use sw_domain::repositories::user::UserRepository;
use sw_domain::value_objects::ids::UserId;
use sw_domain::value_objects::Email;

/// SQLx-backed implementation of [`UserRepository`].
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO users (id, email, name, password_hash, phone, picture_url, age, address, role, status, is_verified, expertise, bio, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"#,
        )
        .bind(user.id.into_uuid())
        .bind(user.email.as_str())
        .bind(&user.name)
        .bind(&user.password_hash)
        .bind(&user.phone)
        .bind(&user.picture_url)
        .bind(user.age)
        .bind(&user.address)
        .bind(user.role.as_str())
        .bind(user.status.as_str())
        .bind(user.is_verified)
        .bind(&user.expertise)
        .bind(&user.bio)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create user: {e}")))?;
        Ok(())
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"SELECT id, email, name, password_hash, phone, picture_url, age, address,
                      role, status, is_verified, expertise, bio, created_at, updated_at
               FROM users WHERE id = $1"#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find user by id: {e}")))?;
        row.map(UserRow::into_domain).transpose()
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"SELECT id, email, name, password_hash, phone, picture_url, age, address,
                      role, status, is_verified, expertise, bio, created_at, updated_at
               FROM users WHERE email = $1"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find user by email: {e}")))?;
        row.map(UserRow::into_domain).transpose()
    }

    async fn update(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE users SET email = $2, name = $3, password_hash = $4, phone = $5,
               picture_url = $6, age = $7, address = $8, role = $9, status = $10,
               is_verified = $11, expertise = $12, bio = $13, updated_at = $14
               WHERE id = $1"#,
        )
        .bind(user.id.into_uuid())
        .bind(user.email.as_str())
        .bind(&user.name)
        .bind(&user.password_hash)
        .bind(&user.phone)
        .bind(&user.picture_url)
        .bind(user.age)
        .bind(&user.address)
        .bind(user.role.as_str())
        .bind(user.status.as_str())
        .bind(user.is_verified)
        .bind(&user.expertise)
        .bind(&user.bio)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to update user: {e}")))?;
        Ok(())
    }

    async fn delete(&self, id: UserId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to delete user: {e}")))?;
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<User>, DomainError> {
        let rows = sqlx::query_as::<_, UserRow>(
            r#"SELECT id, email, name, password_hash, phone, picture_url, age, address,
                      role, status, is_verified, expertise, bio, created_at, updated_at
               FROM users ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to list users: {e}")))?;
        rows.into_iter().map(UserRow::into_domain).collect()
    }
}

/// Raw database row for the `users` table.
#[derive(sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    email: String,
    name: String,
    password_hash: Option<String>,
    phone: Option<String>,
    picture_url: Option<String>,
    age: Option<i16>,
    address: Option<String>,
    role: String,
    status: String,
    is_verified: bool,
    expertise: Option<String>,
    bio: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserRow {
    fn into_domain(self) -> Result<User, DomainError> {
        let role = UserRole::from_str(&self.role).ok_or_else(|| {
            DomainError::infrastructure(format!("invalid user role: {}", self.role))
        })?;
        let status = UserStatus::from_str(&self.status).ok_or_else(|| {
            DomainError::infrastructure(format!("invalid user status: {}", self.status))
        })?;
        let email = Email::new(&self.email)
            .map_err(|_| DomainError::infrastructure("invalid email in database".to_string()))?;

        Ok(User {
            id: UserId::from_uuid(self.id),
            email,
            name: self.name,
            password_hash: self.password_hash,
            phone: self.phone,
            picture_url: self.picture_url,
            age: self.age,
            address: self.address,
            role,
            status,
            is_verified: self.is_verified,
            expertise: self.expertise,
            bio: self.bio,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
