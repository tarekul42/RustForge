use crate::events::DomainEvent;
use crate::value_objects::ids::{CategoryId, LevelId, UserId, WorkshopId, WorkshopImageId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Aggregate root for a workshop (course).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workshop {
    /// Unique identifier for this workshop.
    pub id: WorkshopId,
    /// Human-readable title.
    pub title: String,
    /// URL-safe slug (unique).
    pub slug: String,
    /// Optional detailed description.
    pub description: Option<String>,
    /// Optional physical location.
    pub location: Option<String>,
    /// Price in cents (0 = free).
    pub price_cents: i64,
    /// Optional start date/time.
    pub start_date: Option<DateTime<Utc>>,
    /// Optional end date/time.
    pub end_date: Option<DateTime<Utc>>,
    /// Maximum number of seats (None = unlimited).
    pub max_seats: Option<i32>,
    /// Current number of enrolled students.
    pub current_enrollments: i32,
    /// Minimum age requirement (optional).
    pub min_age: Option<i16>,
    /// Foreign key to the category.
    pub category_id: CategoryId,
    /// Foreign key to the difficulty level.
    pub level_id: LevelId,
    /// User who created this workshop.
    pub created_by: UserId,
    /// Timestamp of creation.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update.
    pub updated_at: DateTime<Utc>,
}

impl Workshop {
    /// Create a new workshop with zero enrollments.
    ///
    /// Returns the workshop along with a `WorkshopCreated` domain event.
    pub fn new(
        title: String,
        slug: String,
        price_cents: i64,
        category_id: CategoryId,
        level_id: LevelId,
        created_by: UserId,
    ) -> (Self, DomainEvent) {
        let now = Utc::now();
        let workshop = Self {
            id: WorkshopId::new(),
            title,
            slug,
            description: None,
            location: None,
            price_cents,
            start_date: None,
            end_date: None,
            max_seats: None,
            current_enrollments: 0,
            min_age: None,
            category_id,
            level_id,
            created_by,
            created_at: now,
            updated_at: now,
        };
        let event = DomainEvent::WorkshopCreated {
            workshop_id: workshop.id,
        };
        (workshop, event)
    }

    /// Reserve one seat, incrementing the enrollment count.
    ///
    /// Returns an error if the workshop is full.
    pub fn reserve_seat(&mut self) -> Result<DomainEvent, crate::error::DomainError> {
        if let Some(max) = self.max_seats {
            if self.current_enrollments >= max {
                return Err(crate::error::DomainError::WorkshopFull);
            }
        }
        self.current_enrollments += 1;
        self.updated_at = Utc::now();
        Ok(DomainEvent::WorkshopUpdated {
            workshop_id: self.id,
        })
    }

    /// Release one seat, decrementing the enrollment count (floor at zero).
    pub fn release_seat(&mut self) {
        self.current_enrollments = (self.current_enrollments - 1).max(0);
        self.updated_at = Utc::now();
    }

    /// Return the number of seats still available, if `max_seats` is set.
    pub fn available_seats(&self) -> Option<i32> {
        self.max_seats
            .map(|max| (max - self.current_enrollments).max(0))
    }
}

/// A photo associated with a workshop, typically stored in S3/MinIO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopImage {
    /// Unique identifier for this image.
    pub id: WorkshopImageId,
    /// The workshop this image belongs to.
    pub workshop_id: WorkshopId,
    /// Public URL for the image.
    pub url: String,
    /// The S3/MinIO object key.
    pub s3_key: String,
    /// When the image was uploaded.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::ids::{CategoryId, LevelId, UserId};

    fn make_workshop() -> Workshop {
        let (workshop, _) = Workshop::new(
            "Rust Basics".to_string(),
            "rust-basics".to_string(),
            5000,
            CategoryId::new(),
            LevelId::new(),
            UserId::new(),
        );
        workshop
    }

    #[test]
    fn new_workshop_has_zero_enrollments() {
        let workshop = make_workshop();
        assert_eq!(workshop.current_enrollments, 0);
    }

    #[test]
    fn reserve_seat_increases_count() {
        let mut workshop = make_workshop();
        workshop.reserve_seat().unwrap();
        assert_eq!(workshop.current_enrollments, 1);
    }

    #[test]
    fn reserve_seat_on_full_workshop_fails() {
        let mut workshop = make_workshop();
        workshop.max_seats = Some(1);
        workshop.reserve_seat().unwrap();
        assert!(workshop.reserve_seat().is_err());
    }

    #[test]
    fn release_seat_decreases_count() {
        let mut workshop = make_workshop();
        workshop.reserve_seat().unwrap();
        workshop.release_seat();
        assert_eq!(workshop.current_enrollments, 0);
    }

    #[test]
    fn available_seats_returns_difference() {
        let mut workshop = make_workshop();
        workshop.max_seats = Some(10);
        workshop.reserve_seat().unwrap();
        assert_eq!(workshop.available_seats(), Some(9));
    }
}
