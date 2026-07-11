/// Email value object with validation.
pub mod email;
/// Newtype ID types (UserId, WorkshopId, etc.).
pub mod ids;
/// Money value object representing an amount in cents (i64).
pub mod money;
/// OTP code value object.
pub mod otp;
/// Transaction ID value object.
pub mod transaction_id;

pub use email::Email;
pub use ids::*;
pub use money::Money;
pub use otp::OtpCode;
pub use transaction_id::TransactionId;
