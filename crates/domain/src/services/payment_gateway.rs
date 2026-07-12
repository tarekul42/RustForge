use serde::Deserialize;
use std::collections::HashMap;

/// Port trait for initiating and validating payments through a gateway.
///
/// The concrete implementation (e.g., SSLCommerz) lives in `infrastructure`.
#[async_trait::async_trait]
pub trait PaymentGateway: Send + Sync {
    /// Initiate a payment session. Returns the gateway's response,
    /// which includes a redirect URL for the user.
    async fn init_payment(
        &self,
        transaction_id: &str,
        amount_cents: i64,
        currency: &str,
        cus_name: &str,
        cus_email: &str,
        cus_phone: &str,
    ) -> Result<GatewayInitResponse, PaymentGatewayError>;

    /// Validate a completed payment using its `val_id`.
    async fn validate_payment(
        &self,
        val_id: &str,
    ) -> Result<GatewayValidationResponse, PaymentGatewayError>;

    /// Verify the IPN callback signature.
    fn verify_ipn_signature(&self, data: &HashMap<String, String>) -> bool;
}

/// Response from a payment gateway init call.
#[derive(Debug, Clone, Deserialize)]
pub struct GatewayInitResponse {
    /// Whether the init was successful.
    pub success: bool,
    /// The URL to redirect the user to for payment.
    pub gateway_url: Option<String>,
    /// The gateway session key.
    pub session_key: Option<String>,
    /// Error message if init failed.
    pub error_message: Option<String>,
}

/// Response from a payment gateway validation call.
#[derive(Debug, Clone, Deserialize)]
pub struct GatewayValidationResponse {
    /// Whether the payment is valid.
    pub is_valid: bool,
    /// The transaction amount as returned by the gateway.
    pub amount: Option<String>,
    /// The currency of the transaction.
    pub currency: Option<String>,
    /// The transaction ID from the gateway.
    pub transaction_id: Option<String>,
    /// Raw gateway response data.
    pub raw_data: serde_json::Value,
}

/// Errors returned by the payment gateway.
#[derive(Debug, thiserror::Error)]
pub enum PaymentGatewayError {
    /// The HTTP request to the gateway failed.
    #[error("Gateway request failed: {0}")]
    RequestFailed(String),
    /// Failed to parse the gateway response.
    #[error("Gateway response parse failed: {0}")]
    ResponseParseFailed(String),
    /// The gateway returned a business-level error.
    #[error("Gateway returned error: {0}")]
    GatewayError(String),
}
