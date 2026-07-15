use crate::events::DomainEvent;
use crate::value_objects::ids::LevelId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Aggregate root for a difficulty level (e.g. "Beginner", "Advanced").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    /// Unique identifier for this level.
    pub(crate) id: LevelId,
    /// Human-readable name.
    pub(crate) name: String,
    /// Timestamp of creation.
    pub(crate) created_at: DateTime<Utc>,
    /// Timestamp of the last update.
    pub(crate) updated_at: DateTime<Utc>,
}

impl Level {
    /// Create a new level with the given name.
    ///
    /// Returns the level along with a `LevelCreated` domain event.
    pub fn new(name: String) -> (Self, DomainEvent) {
        let now = Utc::now();
        let level = Self {
            id: LevelId::new(),
            name,
            created_at: now,
            updated_at: now,
        };
        let event = DomainEvent::LevelCreated { level_id: level.id };
        (level, event)
    }

    /// Rename the level. Returns a `LevelUpdated` domain event.
    pub fn rename(&mut self, name: String) -> DomainEvent {
        self.name = name;
        self.updated_at = Utc::now();
        DomainEvent::LevelUpdated { level_id: self.id }
    }

    /// Unique identifier for this level.
    pub fn id(&self) -> LevelId {
        self.id
    }

    /// Human-readable name.
    pub fn name(&self) -> &str {
        &self.name
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

impl Level {
    /// Restore a level from persisted data (used by infrastructure repos).
    pub fn from_parts(
        id: LevelId,
        name: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            created_at,
            updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_level_has_name() {
        let (level, _) = Level::new("Beginner".to_string());
        assert_eq!(level.name, "Beginner");
    }

    #[test]
    fn rename_updates_name() {
        let mut level = Level::new("Beginner".to_string()).0;
        level.rename("Advanced".to_string());
        assert_eq!(level.name, "Advanced");
    }
}
