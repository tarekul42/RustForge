use crate::error::ApplicationError;
use moka::future::Cache;
use std::time::Duration;
use sw_domain::repositories::stats::{PlatformStats, StatsRepository, WorkshopRating};
use tracing::instrument;

/// Application service for platform statistics with an in-memory cache.
pub struct StatsService<SR: StatsRepository> {
    repo: SR,
    platform_cache: Cache<(), PlatformStats>,
    ratings_cache: Cache<(), Vec<WorkshopRating>>,
}

impl<SR: StatsRepository> StatsService<SR> {
    /// Create a new `StatsService`.
    ///
    /// The cache has a TTL of 5 minutes and a max capacity of 10 entries.
    pub fn new(repo: SR) -> Self {
        Self {
            repo,
            platform_cache: Cache::builder()
                .time_to_live(Duration::from_secs(300))
                .max_capacity(1)
                .build(),
            ratings_cache: Cache::builder()
                .time_to_live(Duration::from_secs(300))
                .max_capacity(1)
                .build(),
        }
    }

    /// Get aggregate platform statistics (cached for 5 minutes).
    #[instrument(skip(self))]
    pub async fn platform_stats(&self) -> Result<PlatformStats, ApplicationError> {
        if let Some(stats) = self.platform_cache.get(&()).await {
            return Ok(stats);
        }
        let stats = self
            .repo
            .platform_stats()
            .await
            .map_err(ApplicationError::from)?;
        self.platform_cache.insert((), stats.clone()).await;
        Ok(stats)
    }

    /// Get rating summaries for all workshops (cached for 5 minutes).
    #[instrument(skip(self))]
    pub async fn workshop_ratings(&self) -> Result<Vec<WorkshopRating>, ApplicationError> {
        if let Some(ratings) = self.ratings_cache.get(&()).await {
            return Ok(ratings);
        }
        let ratings = self
            .repo
            .workshop_ratings()
            .await
            .map_err(ApplicationError::from)?;
        self.ratings_cache.insert((), ratings.clone()).await;
        Ok(ratings)
    }
}
