use crate::events::DomainEvent;
use crate::value_objects::ids::{ReviewId, UserId, WorkshopId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The moderation status of a review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    /// Awaiting moderator approval.
    Pending,
    /// Approved and visible to the public.
    Approved,
    /// Rejected and hidden from the public.
    Rejected,
}

impl ReviewStatus {
    /// Return the lowercase string representation of this status.
    #[allow(clippy::should_implement_trait)]
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewStatus::Pending => "pending",
            ReviewStatus::Approved => "approved",
            ReviewStatus::Rejected => "rejected",
        }
    }

    /// Parse a status from its lowercase string representation.
    /// Returns `None` for unknown strings.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(ReviewStatus::Pending),
            "approved" => Some(ReviewStatus::Approved),
            "rejected" => Some(ReviewStatus::Rejected),
            _ => None,
        }
    }
}

/// Aggregate root for a review (user feedback on a workshop).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// Unique identifier for this review.
    pub(crate) id: ReviewId,
    /// The user who wrote the review.
    pub(crate) user_id: UserId,
    /// The workshop being reviewed.
    pub(crate) workshop_id: WorkshopId,
    /// Rating from 1 to 5.
    pub(crate) rating: i16,
    /// Short title (max 120 characters).
    pub(crate) title: String,
    /// Full review text (max 2000 characters).
    pub(crate) content: String,
    /// Moderation status.
    pub(crate) status: ReviewStatus,
    /// Timestamp of creation.
    pub(crate) created_at: DateTime<Utc>,
    /// Timestamp of the last update.
    pub(crate) updated_at: DateTime<Utc>,
}

impl Review {
    /// Create a new review with Pending status.
    ///
    /// Validates that rating is between 1 and 5, title ≤ 120 chars,
    /// and content ≤ 2000 chars.
    ///
    /// Returns the review along with a `ReviewCreated` domain event.
    pub fn new(
        user_id: UserId,
        workshop_id: WorkshopId,
        rating: i16,
        title: String,
        content: String,
    ) -> Result<(Self, DomainEvent), crate::error::DomainError> {
        if !(1..=5).contains(&rating) {
            return Err(crate::error::DomainError::validation(
                "Rating must be between 1 and 5",
            ));
        }
        if title.len() > 120 {
            return Err(crate::error::DomainError::validation(
                "Title must be 120 characters or less",
            ));
        }
        if content.len() > 2000 {
            return Err(crate::error::DomainError::validation(
                "Content must be 2000 characters or less",
            ));
        }
        let now = Utc::now();
        let review = Self {
            id: ReviewId::new(),
            user_id,
            workshop_id,
            rating,
            title,
            content,
            status: ReviewStatus::Pending,
            created_at: now,
            updated_at: now,
        };
        let event = DomainEvent::ReviewCreated {
            review_id: review.id,
        };
        Ok((review, event))
    }

    /// Approve a pending review.
    ///
    /// Returns an error if the review is not currently Pending.
    pub fn approve(&mut self) -> Result<DomainEvent, crate::error::DomainError> {
        match self.status {
            ReviewStatus::Pending => {
                self.status = ReviewStatus::Approved;
                self.updated_at = Utc::now();
                Ok(DomainEvent::ReviewModerated {
                    review_id: self.id,
                    from: "pending",
                    to: "approved",
                })
            }
            _ => Err(crate::error::DomainError::invalid_transition(
                self.status.as_str(),
                "approved",
            )),
        }
    }

    /// Reject a pending review.
    ///
    /// Returns an error if the review is not currently Pending.
    pub fn reject(&mut self) -> Result<DomainEvent, crate::error::DomainError> {
        match self.status {
            ReviewStatus::Pending => {
                self.status = ReviewStatus::Rejected;
                self.updated_at = Utc::now();
                Ok(DomainEvent::ReviewModerated {
                    review_id: self.id,
                    from: "pending",
                    to: "rejected",
                })
            }
            _ => Err(crate::error::DomainError::invalid_transition(
                self.status.as_str(),
                "rejected",
            )),
        }
    }

    // --- Getters ---

    /// Unique identifier for this review.
    pub fn id(&self) -> ReviewId {
        self.id
    }

    /// The user who wrote the review.
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    /// The workshop being reviewed.
    pub fn workshop_id(&self) -> WorkshopId {
        self.workshop_id
    }

    /// Rating from 1 to 5.
    pub fn rating(&self) -> i16 {
        self.rating
    }

    /// Short title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Full review text.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Moderation status.
    pub fn status(&self) -> ReviewStatus {
        self.status
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

    /// Set the rating.
    pub fn set_rating(&mut self, rating: i16) {
        self.rating = rating;
    }

    /// Set the title.
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Set the content.
    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    /// Set the updated_at timestamp to now.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl Review {
    /// Restore a review from persisted data (used by infrastructure repos).
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        id: ReviewId,
        user_id: UserId,
        workshop_id: WorkshopId,
        rating: i16,
        title: String,
        content: String,
        status: ReviewStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            workshop_id,
            rating,
            title,
            content,
            status,
            created_at,
            updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::ids::{UserId, WorkshopId};

    fn make_review() -> Review {
        let (review, _) = Review::new(
            UserId::new(),
            WorkshopId::new(),
            5,
            "Great workshop".to_string(),
            "Really enjoyed it!".to_string(),
        )
        .unwrap();
        review
    }

    #[test]
    fn new_review_is_pending() {
        let review = make_review();
        assert_eq!(review.status, ReviewStatus::Pending);
    }

    #[test]
    fn approve_pending_succeeds() {
        let mut review = make_review();
        review.approve().unwrap();
        assert_eq!(review.status, ReviewStatus::Approved);
    }

    #[test]
    fn reject_pending_succeeds() {
        let mut review = make_review();
        review.reject().unwrap();
        assert_eq!(review.status, ReviewStatus::Rejected);
    }

    #[test]
    fn approve_approved_fails() {
        let mut review = make_review();
        review.approve().unwrap();
        assert!(review.approve().is_err());
    }

    #[test]
    fn invalid_rating_fails() {
        let result = Review::new(
            UserId::new(),
            WorkshopId::new(),
            6,
            "Title".to_string(),
            "Content".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn title_too_long_fails() {
        let result = Review::new(
            UserId::new(),
            WorkshopId::new(),
            5,
            "x".repeat(121),
            "Content".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn reject_approved_fails() {
        let mut review = make_review();
        review.approve().unwrap();
        assert!(review.reject().is_err());
    }

    #[test]
    fn content_too_long_fails() {
        let result = Review::new(
            UserId::new(),
            WorkshopId::new(),
            5,
            "Title".to_string(),
            "x".repeat(2001),
        );
        assert!(result.is_err());
    }

    #[test]
    fn rating_below_minimum_fails() {
        let result = Review::new(
            UserId::new(),
            WorkshopId::new(),
            0,
            "Title".to_string(),
            "Content".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn new_review_returns_created_event() {
        let (_, event) = Review::new(
            UserId::new(),
            WorkshopId::new(),
            4,
            "Nice".to_string(),
            "Good content".to_string(),
        )
        .unwrap();
        assert!(matches!(event, DomainEvent::ReviewCreated { .. }));
    }

    #[test]
    fn approve_returns_moderated_event() {
        let mut review = make_review();
        let event = review.approve().unwrap();
        assert!(matches!(
            event,
            DomainEvent::ReviewModerated { to: "approved", .. }
        ));
    }

    #[test]
    fn reject_returns_moderated_event() {
        let mut review = make_review();
        let event = review.reject().unwrap();
        assert!(matches!(
            event,
            DomainEvent::ReviewModerated { to: "rejected", .. }
        ));
    }
}
