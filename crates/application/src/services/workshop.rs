use crate::error::ApplicationError;
use sw_domain::aggregates::workshop::{Workshop, WorkshopImage};
use sw_domain::events::{DomainEvent, EventStore};
use sw_domain::repositories::category::CategoryRepository;
use sw_domain::repositories::level::LevelRepository;
use sw_domain::repositories::workshop::WorkshopRepository;
use sw_domain::value_objects::ids::{CategoryId, LevelId, UserId, WorkshopId, WorkshopImageId};
use tracing::instrument;

/// Input for creating a workshop.
#[derive(Debug)]
pub struct CreateWorkshopInput {
    /// Workshop title.
    pub title: String,
    /// URL-safe unique slug.
    pub slug: String,
    /// Price in cents (0 = free).
    pub price_cents: i64,
    /// FK to the category.
    pub category_id: CategoryId,
    /// FK to the difficulty level.
    pub level_id: LevelId,
    /// User creating the workshop.
    pub created_by: UserId,
    /// Optional description.
    pub description: Option<String>,
    /// Optional physical location.
    pub location: Option<String>,
    /// Optional start date/time.
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Optional end date/time.
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Optional max seats.
    pub max_seats: Option<i32>,
    /// Optional minimum age.
    pub min_age: Option<i16>,
}

/// Input for updating a workshop. `None` fields mean "leave unchanged".
#[derive(Debug)]
pub struct UpdateWorkshopInput {
    /// Workshop to update.
    pub id: WorkshopId,
    /// New title.
    pub title: Option<String>,
    /// New slug.
    pub slug: Option<String>,
    /// New description.
    pub description: Option<String>,
    /// New location.
    pub location: Option<String>,
    /// New price in cents.
    pub price_cents: Option<i64>,
    /// New category FK.
    pub category_id: Option<CategoryId>,
    /// New level FK.
    pub level_id: Option<LevelId>,
    /// New start date (`Some(None)` to clear).
    pub start_date: Option<Option<chrono::DateTime<chrono::Utc>>>,
    /// New end date (`Some(None)` to clear).
    pub end_date: Option<Option<chrono::DateTime<chrono::Utc>>>,
    /// New max seats (`Some(None)` to clear).
    pub max_seats: Option<Option<i32>>,
    /// New min age (`Some(None)` to clear).
    pub min_age: Option<Option<i16>>,
}

/// Application service for workshop CRUD and image management with audit logging.
pub struct WorkshopService<
    W: WorkshopRepository,
    C: CategoryRepository,
    L: LevelRepository,
    E: EventStore,
> {
    repo: W,
    category_repo: C,
    level_repo: L,
    event_store: E,
}

impl<W: WorkshopRepository, C: CategoryRepository, L: LevelRepository, E: EventStore>
    WorkshopService<W, C, L, E>
{
    /// Create a new `WorkshopService`.
    pub fn new(repo: W, category_repo: C, level_repo: L, event_store: E) -> Self {
        Self {
            repo,
            category_repo,
            level_repo,
            event_store,
        }
    }

    /// Create a new workshop.
    #[instrument(skip(self))]
    pub async fn create(&self, input: CreateWorkshopInput) -> Result<Workshop, ApplicationError> {
        if self.repo.find_by_slug(&input.slug).await?.is_some() {
            return Err(ApplicationError::conflict(format!(
                "Workshop with slug '{}' already exists",
                input.slug
            )));
        }

        let _category = self
            .category_repo
            .find_by_id(input.category_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Category", input.category_id))?;

        let _level = self
            .level_repo
            .find_by_id(input.level_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Level", input.level_id))?;

        let (mut workshop, event) = Workshop::new(
            input.title,
            input.slug,
            input.price_cents,
            input.category_id,
            input.level_id,
            input.created_by,
        );
        workshop.description = input.description;
        workshop.location = input.location;
        workshop.start_date = input.start_date;
        workshop.end_date = input.end_date;
        workshop.max_seats = input.max_seats;
        workshop.min_age = input.min_age;

        self.repo.create(&workshop).await?;
        self.publish_event(event).await?;
        Ok(workshop)
    }

    /// Find a workshop by its ID.
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: WorkshopId) -> Result<Workshop, ApplicationError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Workshop", id))
    }

    /// Find a workshop by its slug.
    #[instrument(skip(self))]
    pub async fn get_by_slug(&self, slug: &str) -> Result<Workshop, ApplicationError> {
        self.repo
            .find_by_slug(slug)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Workshop", slug))
    }

    /// Update a workshop's fields.
    #[instrument(skip(self))]
    pub async fn update(&self, input: UpdateWorkshopInput) -> Result<Workshop, ApplicationError> {
        let mut workshop = self
            .repo
            .find_by_id(input.id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Workshop", input.id))?;

        if let Some(title) = input.title {
            workshop.title = title;
        }
        if let Some(slug) = input.slug {
            workshop.slug = slug;
        }
        if let Some(description) = input.description {
            workshop.description = Some(description);
        }
        if let Some(location) = input.location {
            workshop.location = Some(location);
        }
        if let Some(price_cents) = input.price_cents {
            workshop.price_cents = price_cents;
        }
        if let Some(category_id) = input.category_id {
            let _category = self
                .category_repo
                .find_by_id(category_id)
                .await?
                .ok_or_else(|| ApplicationError::not_found("Category", category_id))?;
            workshop.category_id = category_id;
        }
        if let Some(level_id) = input.level_id {
            let _level = self
                .level_repo
                .find_by_id(level_id)
                .await?
                .ok_or_else(|| ApplicationError::not_found("Level", level_id))?;
            workshop.level_id = level_id;
        }
        if let Some(start_date) = input.start_date {
            workshop.start_date = start_date;
        }
        if let Some(end_date) = input.end_date {
            workshop.end_date = end_date;
        }
        if let Some(max_seats) = input.max_seats {
            workshop.max_seats = max_seats;
        }
        if let Some(min_age) = input.min_age {
            workshop.min_age = min_age;
        }

        workshop.updated_at = chrono::Utc::now();
        self.repo.update(&workshop).await?;
        self.publish_event(DomainEvent::WorkshopUpdated {
            workshop_id: input.id,
        })
        .await?;
        Ok(workshop)
    }

    /// Delete a workshop by ID.
    #[instrument(skip(self))]
    pub async fn delete(&self, id: WorkshopId) -> Result<(), ApplicationError> {
        let _workshop = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Workshop", id))?;

        self.repo.delete(id).await?;
        self.publish_event(DomainEvent::WorkshopDeleted { workshop_id: id })
            .await?;
        Ok(())
    }

    /// Get all images for a workshop.
    #[instrument(skip(self))]
    pub async fn get_images(
        &self,
        workshop_id: WorkshopId,
    ) -> Result<Vec<WorkshopImage>, ApplicationError> {
        self.repo
            .get_images(workshop_id)
            .await
            .map_err(ApplicationError::from)
    }

    /// Add an image to a workshop.
    #[instrument(skip(self))]
    pub async fn add_image(
        &self,
        workshop_id: WorkshopId,
        url: &str,
        s3_key: &str,
    ) -> Result<WorkshopImage, ApplicationError> {
        let _workshop = self
            .repo
            .find_by_id(workshop_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Workshop", workshop_id))?;

        self.repo
            .add_image(workshop_id, url, s3_key)
            .await
            .map_err(ApplicationError::from)
    }

    /// Remove an image from a workshop.
    #[instrument(skip(self))]
    pub async fn remove_image(&self, image_id: WorkshopImageId) -> Result<(), ApplicationError> {
        self.repo
            .remove_image(image_id)
            .await
            .map_err(ApplicationError::from)
    }

    async fn publish_event(&self, event: DomainEvent) -> Result<(), ApplicationError> {
        self.event_store
            .publish(&event)
            .await
            .map_err(|e| ApplicationError::internal(format!("failed to publish event: {e}")))
    }
}
