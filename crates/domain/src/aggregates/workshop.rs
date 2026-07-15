use crate::events::DomainEvent;
use crate::value_objects::ids::{CategoryId, LevelId, UserId, WorkshopId, WorkshopImageId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Aggregate root for a workshop (course).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workshop {
    /// Unique identifier for this workshop.
    pub(crate) id: WorkshopId,
    /// Human-readable title.
    pub(crate) title: String,
    /// URL-safe slug (unique).
    pub(crate) slug: String,
    /// Optional detailed description.
    pub(crate) description: Option<String>,
    /// Optional physical location.
    pub(crate) location: Option<String>,
    /// Price in cents (0 = free).
    pub(crate) price_cents: i64,
    /// Optional start date/time.
    pub(crate) start_date: Option<DateTime<Utc>>,
    /// Optional end date/time.
    pub(crate) end_date: Option<DateTime<Utc>>,
    /// Maximum number of seats (None = unlimited).
    pub(crate) max_seats: Option<i32>,
    /// Current number of enrolled students.
    pub(crate) current_enrollments: i32,
    /// Minimum age requirement (optional).
    pub(crate) min_age: Option<i16>,
    /// Foreign key to the category.
    pub(crate) category_id: CategoryId,
    /// Foreign key to the difficulty level.
    pub(crate) level_id: LevelId,
    /// User who created this workshop.
    pub(crate) created_by: UserId,
    /// Timestamp of creation.
    pub(crate) created_at: DateTime<Utc>,
    /// Timestamp of the last update.
    pub(crate) updated_at: DateTime<Utc>,
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

    // --- Getters ---

    /// Unique identifier for this workshop.
    pub fn id(&self) -> WorkshopId {
        self.id
    }

    /// Human-readable title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// URL-safe slug (unique).
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Optional detailed description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Optional physical location.
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    /// Price in cents (0 = free).
    pub fn price_cents(&self) -> i64 {
        self.price_cents
    }

    /// Optional start date/time.
    pub fn start_date(&self) -> Option<DateTime<Utc>> {
        self.start_date
    }

    /// Optional end date/time.
    pub fn end_date(&self) -> Option<DateTime<Utc>> {
        self.end_date
    }

    /// Maximum number of seats (None = unlimited).
    pub fn max_seats(&self) -> Option<i32> {
        self.max_seats
    }

    /// Current number of enrolled students.
    pub fn current_enrollments(&self) -> i32 {
        self.current_enrollments
    }

    /// Minimum age requirement (optional).
    pub fn min_age(&self) -> Option<i16> {
        self.min_age
    }

    /// Foreign key to the category.
    pub fn category_id(&self) -> CategoryId {
        self.category_id
    }

    /// Foreign key to the difficulty level.
    pub fn level_id(&self) -> LevelId {
        self.level_id
    }

    /// User who created this workshop.
    pub fn created_by(&self) -> UserId {
        self.created_by
    }

    /// Timestamp of creation.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Timestamp of the last update.
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    // --- Setters ---

    /// Set the title.
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Set the slug.
    pub fn set_slug(&mut self, slug: String) {
        self.slug = slug;
    }

    /// Set the description.
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    /// Set the location.
    pub fn set_location(&mut self, location: Option<String>) {
        self.location = location;
    }

    /// Set the price in cents.
    pub fn set_price_cents(&mut self, price_cents: i64) {
        self.price_cents = price_cents;
    }

    /// Set the start date.
    pub fn set_start_date(&mut self, start_date: Option<DateTime<Utc>>) {
        self.start_date = start_date;
    }

    /// Set the end date.
    pub fn set_end_date(&mut self, end_date: Option<DateTime<Utc>>) {
        self.end_date = end_date;
    }

    /// Set the maximum number of seats.
    pub fn set_max_seats(&mut self, max_seats: Option<i32>) {
        self.max_seats = max_seats;
    }

    /// Set the minimum age requirement.
    pub fn set_min_age(&mut self, min_age: Option<i16>) {
        self.min_age = min_age;
    }

    /// Set the category.
    pub fn set_category_id(&mut self, category_id: CategoryId) {
        self.category_id = category_id;
    }

    /// Set the level.
    pub fn set_level_id(&mut self, level_id: LevelId) {
        self.level_id = level_id;
    }

    /// Set the updated_at timestamp to now.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl Workshop {
    /// Restore a workshop from persisted data (used by infrastructure repos).
    pub fn from_parts(
        id: WorkshopId,
        title: String,
        slug: String,
        description: Option<String>,
        location: Option<String>,
        price_cents: i64,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        max_seats: Option<i32>,
        current_enrollments: i32,
        min_age: Option<i16>,
        category_id: CategoryId,
        level_id: LevelId,
        created_by: UserId,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            title,
            slug,
            description,
            location,
            price_cents,
            start_date,
            end_date,
            max_seats,
            current_enrollments,
            min_age,
            category_id,
            level_id,
            created_by,
            created_at,
            updated_at,
        }
    }
}

/// A photo associated with a workshop, typically stored in S3/MinIO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopImage {
    /// Unique identifier for this image.
    pub(crate) id: WorkshopImageId,
    /// The workshop this image belongs to.
    pub(crate) workshop_id: WorkshopId,
    /// Public URL for the image.
    pub(crate) url: String,
    /// The S3/MinIO object key.
    pub(crate) s3_key: String,
    /// When the image was uploaded.
    pub(crate) created_at: DateTime<Utc>,
}

impl WorkshopImage {
    /// Unique identifier for this image.
    pub fn id(&self) -> WorkshopImageId {
        self.id
    }

    /// The workshop this image belongs to.
    pub fn workshop_id(&self) -> WorkshopId {
        self.workshop_id
    }

    /// Public URL for the image.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The S3/MinIO object key.
    pub fn s3_key(&self) -> &str {
        &self.s3_key
    }

    /// When the image was uploaded.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Restore a workshop image from persisted data (used by infrastructure repos).
    pub fn from_parts(
        id: WorkshopImageId,
        workshop_id: WorkshopId,
        url: String,
        s3_key: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workshop_id,
            url,
            s3_key,
            created_at,
        }
    }
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

    #[test]
    fn reserve_seat_on_unlimited_workshop_succeeds() {
        let mut workshop = make_workshop();
        workshop.max_seats = None;
        for _ in 0..100 {
            workshop.reserve_seat().unwrap();
        }
        assert_eq!(workshop.current_enrollments, 100);
    }

    #[test]
    fn reserve_seat_at_exact_capacity_fails() {
        let mut workshop = make_workshop();
        workshop.max_seats = Some(5);
        for _ in 0..5 {
            workshop.reserve_seat().unwrap();
        }
        assert!(workshop.reserve_seat().is_err());
    }

    #[test]
    fn release_seat_below_zero_floors() {
        let mut workshop = make_workshop();
        workshop.release_seat();
        assert_eq!(workshop.current_enrollments, 0);
    }

    #[test]
    fn available_seats_none_for_unlimited() {
        let workshop = make_workshop();
        assert_eq!(workshop.available_seats(), None);
    }

    #[test]
    fn available_seats_zero_when_full() {
        let mut workshop = make_workshop();
        workshop.max_seats = Some(3);
        for _ in 0..3 {
            workshop.reserve_seat().unwrap();
        }
        assert_eq!(workshop.available_seats(), Some(0));
    }

    #[test]
    fn new_workshop_returns_created_event() {
        let (_, event) = Workshop::new(
            "Test".to_string(),
            "test".to_string(),
            1000,
            CategoryId::new(),
            LevelId::new(),
            UserId::new(),
        );
        assert!(matches!(event, DomainEvent::WorkshopCreated { .. }));
    }
}
