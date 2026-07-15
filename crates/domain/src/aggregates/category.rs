use crate::events::DomainEvent;
use crate::value_objects::ids::CategoryId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Aggregate root for a workshop category (e.g. "Programming", "Design").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier for this category.
    pub(crate) id: CategoryId,
    /// Human-readable name.
    pub(crate) name: String,
    /// URL-safe unique slug.
    pub(crate) slug: String,
    /// Optional description of the category.
    pub(crate) description: Option<String>,
    /// Optional URL to a thumbnail image.
    pub(crate) thumbnail_url: Option<String>,
    /// Timestamp of creation.
    pub(crate) created_at: DateTime<Utc>,
    /// Timestamp of the last update.
    pub(crate) updated_at: DateTime<Utc>,
}

impl Category {
    /// Create a new category with the given name, slug, and optional description/thumbnail.
    ///
    /// Returns the category along with a `CategoryCreated` domain event.
    pub fn new(
        name: String,
        slug: String,
        description: Option<String>,
        thumbnail_url: Option<String>,
    ) -> (Self, DomainEvent) {
        let now = Utc::now();
        let category = Self {
            id: CategoryId::new(),
            name,
            slug,
            description,
            thumbnail_url,
            created_at: now,
            updated_at: now,
        };
        let event = DomainEvent::CategoryCreated {
            category_id: category.id,
        };
        (category, event)
    }

    /// Update category fields. `None` fields are left unchanged.
    pub fn update(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        thumbnail_url: Option<String>,
    ) -> DomainEvent {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(description) = description {
            self.description = Some(description);
        }
        if let Some(thumbnail_url) = thumbnail_url {
            self.thumbnail_url = Some(thumbnail_url);
        }
        self.updated_at = Utc::now();
        DomainEvent::CategoryUpdated {
            category_id: self.id,
        }
    }

    /// Unique identifier for this category.
    pub fn id(&self) -> CategoryId {
        self.id
    }

    /// Human-readable name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// URL-safe unique slug.
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Optional description of the category.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Optional URL to a thumbnail image.
    pub fn thumbnail_url(&self) -> Option<&str> {
        self.thumbnail_url.as_deref()
    }

    /// Timestamp of creation.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Timestamp of the last update.
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

impl Category {
    /// Restore a category from persisted data (used by infrastructure repos).
    pub fn from_parts(
        id: CategoryId,
        name: String,
        slug: String,
        description: Option<String>,
        thumbnail_url: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            slug,
            description,
            thumbnail_url,
            created_at,
            updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_category() -> Category {
        let (category, _) = Category::new(
            "Programming".to_string(),
            "programming".to_string(),
            None,
            None,
        );
        category
    }

    #[test]
    fn new_category_has_defaults() {
        let category = make_category();
        assert_eq!(category.name, "Programming");
        assert_eq!(category.slug, "programming");
        assert!(category.description.is_none());
        assert!(category.thumbnail_url.is_none());
    }

    #[test]
    fn update_changes_fields() {
        let mut category = make_category();
        category.update(
            Some("Coding".to_string()),
            Some("Coding courses".to_string()),
            None,
        );
        assert_eq!(category.name, "Coding");
        assert_eq!(category.description.unwrap(), "Coding courses");
    }
}
