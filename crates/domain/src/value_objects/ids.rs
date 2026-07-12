use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident, $singular:literal) => {
        #[doc = concat!("A strongly-typed identifier for a ", $singular, ".")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            #[doc = concat!("Create a new `", stringify!($name), "` backed by `uuid::Uuid::now_v7()`.")]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            #[doc = concat!("Create a `", stringify!($name), "` from an existing `uuid::Uuid`.")]
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            #[doc = concat!("Borrow the inner `uuid::Uuid` of this `", stringify!($name), "`.")]
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            #[doc = concat!("Consume this `", stringify!($name), "` and return the inner `uuid::Uuid`.")]
            pub fn into_uuid(self) -> Uuid {
                self.0
            }

            #[doc = concat!("Parse a `", stringify!($name), "` from a UUID string.")]
            pub fn parse_str(s: &str) -> Result<Self, uuid::Error> {
                Uuid::parse_str(s).map(Self)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<Uuid> for $name {
            fn from(id: Uuid) -> Self {
                Self(id)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

id_type!(UserId, "user");
id_type!(WorkshopId, "workshop");
id_type!(WorkshopImageId, "workshop_image");
id_type!(EnrollmentId, "enrollment");
id_type!(PaymentId, "payment");
id_type!(ReviewId, "review");
id_type!(SessionId, "session");
id_type!(CategoryId, "category");
id_type!(LevelId, "level");
id_type!(OtpCodeId, "otp_code");
id_type!(OAuthStateId, "oauth_state");
id_type!(AuthCodeId, "auth_code");
id_type!(ContactId, "contact");
id_type!(JobId, "job");
id_type!(RefundLogId, "refund_log");
