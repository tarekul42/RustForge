use crate::events::DomainEvent;
use crate::value_objects::Email;
use crate::value_objects::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The role assigned to a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    /// Super administrator with full system access.
    SuperAdmin,
    /// Administrator with elevated privileges.
    Admin,
    /// Instructor who can create and manage workshops.
    Instructor,
    /// Regular student / attendee.
    Student,
}

impl UserRole {
    /// Return the snake_case string representation of this role.
    #[allow(clippy::should_implement_trait)]
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::SuperAdmin => "super_admin",
            UserRole::Admin => "admin",
            UserRole::Instructor => "instructor",
            UserRole::Student => "student",
        }
    }

    /// Parse a role from its snake_case string representation.
    /// Returns `None` for unknown strings.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "super_admin" => Some(UserRole::SuperAdmin),
            "admin" => Some(UserRole::Admin),
            "instructor" => Some(UserRole::Instructor),
            "student" => Some(UserRole::Student),
            _ => None,
        }
    }
}

/// The status of a user account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    /// Account is active and usable.
    Active,
    /// Account is inactive (e.g. user disabled it).
    Inactive,
    /// Account has been blocked by an administrator.
    Blocked,
}

impl UserStatus {
    /// Return the lowercase string representation of this status.
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Inactive => "inactive",
            UserStatus::Blocked => "blocked",
        }
    }

    /// Parse a status from its lowercase string representation.
    /// Returns `None` for unknown strings.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(UserStatus::Active),
            "inactive" => Some(UserStatus::Inactive),
            "blocked" => Some(UserStatus::Blocked),
            _ => None,
        }
    }
}

/// Aggregate root for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier for this user.
    pub id: UserId,
    /// Verified email address (lowercased).
    pub email: Email,
    /// Display name.
    pub name: String,
    /// Bcrypt hash of the user's password.
    pub password_hash: Option<String>,
    /// Optional phone number.
    pub phone: Option<String>,
    /// URL to the user's profile picture.
    pub picture_url: Option<String>,
    /// Age in years.
    pub age: Option<i16>,
    /// Physical / mailing address.
    pub address: Option<String>,
    /// System role (affects permissions).
    pub role: UserRole,
    /// Account status.
    pub status: UserStatus,
    /// Whether the email address has been verified.
    pub is_verified: bool,
    /// Instructor expertise description.
    pub expertise: Option<String>,
    /// Short biography.
    pub bio: Option<String>,
    /// Timestamp of account creation.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update.
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user with default role (Student) and active status.
    ///
    /// Returns the user along with a `UserRegistered` domain event.
    pub fn new(email: Email, name: String, password_hash: Option<String>) -> (Self, DomainEvent) {
        let now = Utc::now();
        let user = Self {
            id: UserId::new(),
            email,
            name,
            password_hash,
            phone: None,
            picture_url: None,
            age: None,
            address: None,
            role: UserRole::Student,
            status: UserStatus::Active,
            is_verified: false,
            expertise: None,
            bio: None,
            created_at: now,
            updated_at: now,
        };
        let event = DomainEvent::UserRegistered {
            user_id: user.id,
            email: user.email.clone(),
        };
        (user, event)
    }

    /// Mark the user's email as verified.
    ///
    /// Returns an error if the user is already verified.
    pub fn verify(&mut self) -> Result<DomainEvent, crate::error::DomainError> {
        if self.is_verified {
            return Err(crate::error::DomainError::invalid_transition(
                "verified", "verified",
            ));
        }
        self.is_verified = true;
        self.updated_at = Utc::now();
        Ok(DomainEvent::UserVerified { user_id: self.id })
    }

    /// Change the user's password hash.
    pub fn change_password(
        &mut self,
        new_password_hash: String,
    ) -> Result<DomainEvent, crate::error::DomainError> {
        self.password_hash = Some(new_password_hash);
        self.updated_at = Utc::now();
        Ok(DomainEvent::PasswordChanged { user_id: self.id })
    }

    /// Update profile fields. `None` fields are left unchanged.
    pub fn update_profile(
        &mut self,
        name: Option<String>,
        phone: Option<String>,
        age: Option<i16>,
        address: Option<String>,
        expertise: Option<String>,
        bio: Option<String>,
    ) -> DomainEvent {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(phone) = phone {
            self.phone = Some(phone);
        }
        if let Some(age) = age {
            self.age = Some(age);
        }
        if let Some(address) = address {
            self.address = Some(address);
        }
        if let Some(expertise) = expertise {
            self.expertise = Some(expertise);
        }
        if let Some(bio) = bio {
            self.bio = Some(bio);
        }
        self.updated_at = Utc::now();
        DomainEvent::UserUpdated { user_id: self.id }
    }

    /// Check if the user has admin-level privileges.
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::SuperAdmin)
    }

    /// Check if the user is a super admin.
    pub fn is_super_admin(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin)
    }

    /// Check if the user can manage workshops (admin, super admin, or instructor).
    pub fn can_manage_workshops(&self) -> bool {
        matches!(
            self.role,
            UserRole::Admin | UserRole::SuperAdmin | UserRole::Instructor
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user() -> User {
        let email = Email::new("test@example.com").unwrap();
        let (user, _) = User::new(email, "Test User".to_string(), None);
        user
    }

    #[test]
    fn new_user_is_unverified() {
        let user = make_user();
        assert!(!user.is_verified);
        assert_eq!(user.role, UserRole::Student);
        assert_eq!(user.status, UserStatus::Active);
    }

    #[test]
    fn verify_user() {
        let mut user = make_user();
        let event = user.verify().unwrap();
        assert!(user.is_verified);
        assert!(matches!(event, DomainEvent::UserVerified { .. }));
    }

    #[test]
    fn verify_twice_fails() {
        let mut user = make_user();
        user.verify().unwrap();
        assert!(user.verify().is_err());
    }

    #[test]
    fn user_is_admin_returns_false_for_student() {
        let user = make_user();
        assert!(!user.is_admin());
    }

    #[test]
    fn change_password_updates_hash() {
        let mut user = make_user();
        assert!(user.password_hash.is_none());
        let event = user.change_password("new_hash".to_string()).unwrap();
        assert_eq!(user.password_hash.as_deref(), Some("new_hash"));
        assert!(matches!(event, DomainEvent::PasswordChanged { .. }));
    }

    #[test]
    fn change_password_to_same_hash_works() {
        let mut user = make_user();
        user.change_password("hash".to_string()).unwrap();
        user.change_password("hash".to_string()).unwrap();
        assert_eq!(user.password_hash.as_deref(), Some("hash"));
    }

    #[test]
    fn update_profile_changes_name() {
        let mut user = make_user();
        user.update_profile(Some("New Name".to_string()), None, None, None, None, None);
        assert_eq!(user.name, "New Name");
    }

    #[test]
    fn update_profile_with_none_keeps_name() {
        let mut user = make_user();
        let original = user.name.clone();
        user.update_profile(None, None, None, None, None, None);
        assert_eq!(user.name, original);
    }

    #[test]
    fn update_profile_changes_all_fields() {
        let mut user = make_user();
        let event = user.update_profile(
            Some("New".to_string()),
            Some("555".to_string()),
            Some(30),
            Some("Addr".to_string()),
            Some("Expert".to_string()),
            Some("Bio".to_string()),
        );
        assert_eq!(user.name, "New");
        assert_eq!(user.phone.as_deref(), Some("555"));
        assert_eq!(user.age, Some(30));
        assert_eq!(user.address.as_deref(), Some("Addr"));
        assert_eq!(user.expertise.as_deref(), Some("Expert"));
        assert_eq!(user.bio.as_deref(), Some("Bio"));
        assert!(matches!(event, DomainEvent::UserUpdated { .. }));
    }

    #[test]
    fn is_admin_true_for_admin() {
        let mut user = make_user();
        user.role = UserRole::Admin;
        assert!(user.is_admin());
    }

    #[test]
    fn is_admin_true_for_super_admin() {
        let mut user = make_user();
        user.role = UserRole::SuperAdmin;
        assert!(user.is_admin());
    }

    #[test]
    fn is_super_admin_false_for_admin() {
        let mut user = make_user();
        user.role = UserRole::Admin;
        assert!(!user.is_super_admin());
    }

    #[test]
    fn is_super_admin_true_for_super_admin() {
        let mut user = make_user();
        user.role = UserRole::SuperAdmin;
        assert!(user.is_super_admin());
    }

    #[test]
    fn can_manage_workshops_true_for_instructor() {
        let mut user = make_user();
        user.role = UserRole::Instructor;
        assert!(user.can_manage_workshops());
    }

    #[test]
    fn can_manage_workshops_false_for_student() {
        let user = make_user();
        assert_eq!(user.role, UserRole::Student);
        assert!(!user.can_manage_workshops());
    }

    #[test]
    fn role_as_str_and_from_str_round_trip() {
        for role in &[
            UserRole::SuperAdmin,
            UserRole::Admin,
            UserRole::Instructor,
            UserRole::Student,
        ] {
            let s = role.as_str();
            assert_eq!(UserRole::from_str(s), Some(*role));
        }
    }

    #[test]
    fn status_as_str_and_from_str_round_trip() {
        for status in &[
            UserStatus::Active,
            UserStatus::Inactive,
            UserStatus::Blocked,
        ] {
            let s = status.as_str();
            assert_eq!(UserStatus::from_str(s), Some(*status));
        }
    }

    #[test]
    fn role_from_str_invalid_returns_none() {
        assert_eq!(UserRole::from_str("unknown"), None);
    }

    #[test]
    fn status_from_str_invalid_returns_none() {
        assert_eq!(UserStatus::from_str("unknown"), None);
    }
}
