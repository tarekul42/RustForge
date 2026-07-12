use crate::error::ApplicationError;
use sw_domain::aggregates::review::{Review, ReviewStatus};
use sw_domain::events::{DomainEvent, EventStore};
use sw_domain::repositories::enrollment::EnrollmentRepository;
use sw_domain::repositories::review::ReviewRepository;
use sw_domain::repositories::workshop::WorkshopRepository;
use sw_domain::value_objects::ids::{ReviewId, UserId, WorkshopId};
use tracing::instrument;

/// Input for creating a new review.
#[derive(Debug)]
pub struct CreateReviewInput {
    /// The user writing the review.
    pub user_id: UserId,
    /// The workshop being reviewed.
    pub workshop_id: WorkshopId,
    /// Rating from 1 to 5.
    pub rating: i16,
    /// Short title (max 120 characters).
    pub title: String,
    /// Full review text (max 2000 characters).
    pub content: String,
}

/// Input for updating an existing review (only allowed while Pending).
#[derive(Debug)]
pub struct UpdateReviewInput {
    /// Review to update.
    pub id: ReviewId,
    /// New rating.
    pub rating: Option<i16>,
    /// New title.
    pub title: Option<String>,
    /// New content.
    pub content: Option<String>,
}

/// Application service for review operations.
pub struct ReviewService<
    RR: ReviewRepository,
    ER: EnrollmentRepository,
    WR: WorkshopRepository,
    ES: EventStore,
> {
    repo: RR,
    enrollment_repo: ER,
    workshop_repo: WR,
    event_store: ES,
}

impl<RR: ReviewRepository, ER: EnrollmentRepository, WR: WorkshopRepository, ES: EventStore>
    ReviewService<RR, ER, WR, ES>
{
    /// Create a new `ReviewService`.
    pub fn new(repo: RR, enrollment_repo: ER, workshop_repo: WR, event_store: ES) -> Self {
        Self {
            repo,
            enrollment_repo,
            workshop_repo,
            event_store,
        }
    }

    /// Create a review for a workshop the user has attended.
    ///
    /// Requires the user to have a completed enrollment for the workshop
    /// and no existing review for the same workshop.
    #[instrument(skip(self))]
    pub async fn create(&self, input: CreateReviewInput) -> Result<Review, ApplicationError> {
        let _workshop = self
            .workshop_repo
            .find_by_id(input.workshop_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Workshop", input.workshop_id))?;

        let enrollments = self
            .enrollment_repo
            .find_by_user_and_workshop(input.user_id, input.workshop_id)
            .await?;
        let has_completed = enrollments.iter().any(|e| {
            matches!(
                e.status,
                sw_domain::aggregates::enrollment::EnrollmentStatus::Complete
            )
        });
        if !has_completed {
            return Err(ApplicationError::validation(
                "You must have a completed enrollment to review this workshop",
            ));
        }

        let existing = self
            .repo
            .find_by_user_and_workshop(input.user_id, input.workshop_id)
            .await?;
        if existing.is_some() {
            return Err(ApplicationError::conflict(
                "You have already reviewed this workshop",
            ));
        }

        let (review, event) = Review::new(
            input.user_id,
            input.workshop_id,
            input.rating,
            input.title,
            input.content,
        )?;

        self.repo.create(&review).await?;
        self.publish_event(event).await?;
        Ok(review)
    }

    /// Find a review by ID.
    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: ReviewId) -> Result<Option<Review>, ApplicationError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(ApplicationError::from)
    }

    /// Find all reviews for a workshop.
    ///
    /// When `only_approved` is true (default for public endpoints), only
    /// approved reviews are returned.
    #[instrument(skip(self))]
    pub async fn find_by_workshop(
        &self,
        workshop_id: WorkshopId,
        only_approved: bool,
    ) -> Result<Vec<Review>, ApplicationError> {
        let reviews = self
            .repo
            .find_by_workshop(workshop_id)
            .await
            .map_err(ApplicationError::from)?;
        if only_approved {
            Ok(reviews
                .into_iter()
                .filter(|r| r.status == ReviewStatus::Approved)
                .collect())
        } else {
            Ok(reviews)
        }
    }

    /// Approve a pending review.
    #[instrument(skip(self))]
    pub async fn approve(&self, id: ReviewId) -> Result<Review, ApplicationError> {
        let mut review = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Review", id))?;

        let event = review.approve()?;
        self.repo.update(&review).await?;
        self.publish_event(event).await?;
        Ok(review)
    }

    /// Reject a pending review.
    #[instrument(skip(self))]
    pub async fn reject(&self, id: ReviewId) -> Result<Review, ApplicationError> {
        let mut review = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Review", id))?;

        let event = review.reject()?;
        self.repo.update(&review).await?;
        self.publish_event(event).await?;
        Ok(review)
    }

    /// Update a review (only allowed while Pending).
    #[instrument(skip(self))]
    pub async fn update(&self, input: UpdateReviewInput) -> Result<Review, ApplicationError> {
        let mut review = self
            .repo
            .find_by_id(input.id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Review", input.id))?;

        if review.status != ReviewStatus::Pending {
            return Err(ApplicationError::validation(
                "Can only update a review while it is pending",
            ));
        }

        if let Some(rating) = input.rating {
            if !(1..=5).contains(&rating) {
                return Err(ApplicationError::validation(
                    "Rating must be between 1 and 5",
                ));
            }
            review.rating = rating;
        }
        if let Some(title) = input.title {
            if title.len() > 120 {
                return Err(ApplicationError::validation(
                    "Title must be 120 characters or less",
                ));
            }
            review.title = title;
        }
        if let Some(content) = input.content {
            if content.len() > 2000 {
                return Err(ApplicationError::validation(
                    "Content must be 2000 characters or less",
                ));
            }
            review.content = content;
        }

        review.updated_at = chrono::Utc::now();
        self.repo.update(&review).await?;
        Ok(review)
    }

    /// Delete a review by ID.
    #[instrument(skip(self))]
    pub async fn delete(&self, id: ReviewId) -> Result<(), ApplicationError> {
        let _review = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Review", id))?;

        self.repo.delete(id).await?;
        Ok(())
    }

    async fn publish_event(&self, event: DomainEvent) -> Result<(), ApplicationError> {
        self.event_store
            .publish(&event, None)
            .await
            .map_err(|e| ApplicationError::internal(format!("failed to publish event: {e}")))
    }
}
