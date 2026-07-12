use crate::error::DomainError;
use crate::value_objects::ids::WorkshopId;
use serde::{Deserialize, Serialize};

/// Aggregate platform statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformStats {
    /// Total number of registered users.
    pub total_users: i64,
    /// Total number of workshops.
    pub total_workshops: i64,
    /// Total number of enrollments (all statuses).
    pub total_enrollments: i64,
    /// Total number of reviews submitted.
    pub total_reviews: i64,
}

/// Per-workshop rating summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopRating {
    /// The workshop ID.
    pub workshop_id: WorkshopId,
    /// Average rating (0.0 if no reviews).
    pub average_rating: f64,
    /// Number of reviews for this workshop.
    pub review_count: i64,
}

/// Repository for read-model statistics queries.
#[async_trait::async_trait]
pub trait StatsRepository: Send + Sync {
    /// Retrieve aggregate platform statistics.
    async fn platform_stats(&self) -> Result<PlatformStats, DomainError>;
    /// Retrieve rating summaries for all workshops.
    async fn workshop_ratings(&self) -> Result<Vec<WorkshopRating>, DomainError>;
}
