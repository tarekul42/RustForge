use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

/// Configuration for the SSLCommerz payment gateway.
#[derive(Debug, Clone)]
pub struct SslCommerzConfig {
    /// Merchant store ID provided by SSLCommerz.
    pub store_id: String,
    /// Merchant store password provided by SSLCommerz.
    pub store_passwd: String,
    /// Base URL for the SSLCommerz API (sandbox or production).
    pub base_url: String,
    /// Success redirect URL (absolute).
    pub success_url: String,
    /// Failure redirect URL (absolute).
    pub fail_url: String,
    /// Cancel redirect URL (absolute).
    pub cancel_url: String,
    /// IPN notification URL (absolute).
    pub ipn_url: String,
}

impl Default for SslCommerzConfig {
    fn default() -> Self {
        Self {
            store_id: String::new(),
            store_passwd: String::new(),
            base_url: "https://sandbox.sslcommerz.com".to_string(),
            success_url: String::new(),
            fail_url: String::new(),
            cancel_url: String::new(),
            ipn_url: String::new(),
        }
    }
}

impl SslCommerzConfig {
    /// The gateway endpoint for initiating a payment session.
    pub fn gateway_url(&self) -> String {
        format!("{}/gwprocess/v4/api.php", self.base_url)
    }

    /// The validation endpoint for verifying a transaction.
    pub fn validation_url(&self, val_id: &str) -> String {
        format!(
            "{}/validator/api/validationserverAPI.php?val_id={}&store_id={}&store_passwd={}&v=1&format=json",
            self.base_url, val_id, self.store_id, self.store_passwd
        )
    }
}

/// Response from the SSLCommerz init API.
#[derive(Debug, Deserialize, Serialize)]
pub struct SslcInitResponse {
    /// Status indicator: "SUCCESS" or "FAILED".
    pub status: String,
    /// If successful, the gateway redirect URL.
    #[serde(default)]
    pub gateway_page_url: Option<String>,
    /// If failed, the error reason.
    #[serde(default)]
    pub failedreason: Option<String>,
    /// Unique session key from the gateway.
    #[serde(default)]
    pub sessionkey: Option<String>,
    /// Transaction reference.
    #[serde(default, rename = "tran_id")]
    pub transaction_id: Option<String>,
}

/// Response from the SSLCommerz validation API.
#[derive(Debug, Deserialize, Serialize)]
pub struct SslcValidationResponse {
    /// Status: "VALID", "VALIDATED", or "FAILED".
    pub status: String,
    /// The transaction ID that was validated.
    #[serde(default, rename = "tran_id")]
    pub transaction_id: Option<String>,
    /// The validation ID.
    #[serde(default, rename = "val_id")]
    pub val_id: Option<String>,
    /// The amount that was paid.
    #[serde(default)]
    pub amount: Option<String>,
    /// The currency (e.g., "BDT").
    #[serde(default)]
    pub currency: Option<String>,
    /// The card type used.
    #[serde(default, rename = "card_type")]
    pub card_type: Option<String>,
    /// The bank transaction ID.
    #[serde(default, rename = "bank_tran_id")]
    pub bank_transaction_id: Option<String>,
    /// Raw gateway response data as JSON.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Data received from SSLCommerz via POST (IPN callback or redirect).
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SslcIpnData {
    /// Validation ID.
    #[serde(default, rename = "val_id")]
    pub val_id: Option<String>,
    /// Transaction ID.
    #[serde(default, rename = "tran_id")]
    pub transaction_id: Option<String>,
    /// Payment status from gateway.
    #[serde(default)]
    pub status: Option<String>,
    /// Currency amount.
    #[serde(default)]
    pub amount: Option<String>,
    /// Currency type.
    #[serde(default)]
    pub currency: Option<String>,
    /// The card issuer country.
    #[serde(default)]
    pub card_issuer_country: Option<String>,
    /// Store amount that was settled.
    #[serde(default, rename = "store_amount")]
    pub store_amount: Option<String>,
    /// The verification hash sent by SSLCommerz.
    #[serde(default, rename = "verify_sign")]
    pub verify_sign: Option<String>,
    /// The key used in the verify_sign hash.
    #[serde(default, rename = "verify_key")]
    pub verify_key: Option<String>,
    /// Raw gateway data.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Client for interacting with the SSLCommerz payment gateway.
#[derive(Debug, Clone)]
pub struct SslCommerzClient {
    config: SslCommerzConfig,
    http: reqwest::Client,
}

impl SslCommerzClient {
    /// Create a new SSLCommerz client with the given configuration.
    pub fn new(config: SslCommerzConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    /// Initialize a payment session with SSLCommerz.
    ///
    /// Posts a form-encoded request with order details.
    /// Returns the gateway response, which includes the redirect URL on success.
    pub async fn init_payment(
        &self,
        transaction_id: &str,
        amount_cents: i64,
        currency: &str,
        cus_name: &str,
        cus_email: &str,
        cus_phone: &str,
    ) -> Result<SslcInitResponse, SslCommerzError> {
        let amount_in_decimal = format!("{:.2}", amount_cents as f64 / 100.0);
        let params = [
            ("store_id", self.config.store_id.as_str()),
            ("store_passwd", self.config.store_passwd.as_str()),
            ("total_amount", amount_in_decimal.as_str()),
            ("currency", currency),
            ("tran_id", transaction_id),
            ("success_url", self.config.success_url.as_str()),
            ("fail_url", self.config.fail_url.as_str()),
            ("cancel_url", self.config.cancel_url.as_str()),
            ("ipn_url", self.config.ipn_url.as_str()),
            ("cus_name", cus_name),
            ("cus_email", cus_email),
            ("cus_phone", cus_phone),
            ("multi_card_name", "mastercard,visacard,amex,bank"),
            ("shipping_method", "NO"),
            ("num_of_item", "1"),
            ("product_name", "Workshop Enrollment"),
            ("product_category", "Education"),
            ("product_profile", "general"),
        ];

        let resp = self
            .http
            .post(self.config.gateway_url())
            .form(&params)
            .send()
            .await
            .map_err(|e| SslCommerzError::RequestFailed(e.to_string()))?;

        let init_resp: SslcInitResponse = resp
            .json()
            .await
            .map_err(|e| SslCommerzError::ResponseParseFailed(e.to_string()))?;

        Ok(init_resp)
    }

    /// Validate a payment using the `val_id` from the callback/IPN.
    ///
    /// Returns the validation response from SSLCommerz.
    pub async fn validate_payment(
        &self,
        val_id: &str,
    ) -> Result<SslcValidationResponse, SslCommerzError> {
        let resp = self
            .http
            .get(self.config.validation_url(val_id))
            .send()
            .await
            .map_err(|e| SslCommerzError::RequestFailed(e.to_string()))?;

        let validation: SslcValidationResponse = resp
            .json()
            .await
            .map_err(|e| SslCommerzError::ResponseParseFailed(e.to_string()))?;

        Ok(validation)
    }

    /// Verify the IPN signature from SSLCommerz.
    ///
    /// SSLCommerz sends `verify_sign` and `verify_key` in the IPN callback.
    /// The signature is a SHA-512 hash of `store_passwd` combined with the
    /// values of the keys listed in `verify_key`, joined by `&`.
    fn compute_ipn_signature(&self, data: &SslcIpnData) -> bool {
        let Some(ref verify_sign) = data.verify_sign else {
            return false;
        };
        let Some(ref verify_key) = data.verify_key else {
            return false;
        };

        let keys: Vec<&str> = verify_key.split(',').map(|s| s.trim()).collect();
        let mut message = String::new();

        for (i, key) in keys.iter().enumerate() {
            let value = data
                .extra
                .get(*key)
                .and_then(|v| v.as_str())
                .or(match *key {
                    "val_id" => data.val_id.as_deref(),
                    "tran_id" => data.transaction_id.as_deref(),
                    "status" => data.status.as_deref(),
                    "amount" => data.amount.as_deref(),
                    "currency" => data.currency.as_deref(),
                    "store_amount" => data.store_amount.as_deref(),
                    _ => None,
                })
                .unwrap_or_default();
            if i > 0 {
                message.push('&');
            }
            message.push_str(value);
        }

        let message_with_pass = format!("{}{}", message, self.config.store_passwd);
        let mut hasher = Sha512::new();
        hasher.update(message_with_pass.as_bytes());
        let computed = hex::encode(hasher.finalize());

        computed == verify_sign.to_lowercase()
    }
}

use std::collections::HashMap;
use sw_domain::services::payment_gateway::{
    GatewayInitResponse, GatewayValidationResponse, PaymentGateway, PaymentGatewayError,
};

#[async_trait::async_trait]
impl PaymentGateway for SslCommerzClient {
    async fn init_payment(
        &self,
        transaction_id: &str,
        amount_cents: i64,
        currency: &str,
        cus_name: &str,
        cus_email: &str,
        cus_phone: &str,
    ) -> Result<GatewayInitResponse, PaymentGatewayError> {
        let resp = self
            .init_payment(
                transaction_id,
                amount_cents,
                currency,
                cus_name,
                cus_email,
                cus_phone,
            )
            .await
            .map_err(|e| match e {
                SslCommerzError::RequestFailed(msg) => PaymentGatewayError::RequestFailed(msg),
                SslCommerzError::ResponseParseFailed(msg) => {
                    PaymentGatewayError::ResponseParseFailed(msg)
                }
                SslCommerzError::GatewayError { status, reason } => {
                    PaymentGatewayError::GatewayError(format!("{status}: {reason}"))
                }
            })?;

        Ok(GatewayInitResponse {
            success: resp.status == "SUCCESS",
            gateway_url: resp.gateway_page_url,
            session_key: resp.sessionkey,
            error_message: resp.failedreason,
        })
    }

    async fn validate_payment(
        &self,
        val_id: &str,
    ) -> Result<GatewayValidationResponse, PaymentGatewayError> {
        let resp = self.validate_payment(val_id).await.map_err(|e| match e {
            SslCommerzError::RequestFailed(msg) => PaymentGatewayError::RequestFailed(msg),
            SslCommerzError::ResponseParseFailed(msg) => {
                PaymentGatewayError::ResponseParseFailed(msg)
            }
            SslCommerzError::GatewayError { status, reason } => {
                PaymentGatewayError::GatewayError(format!("{status}: {reason}"))
            }
        })?;

        let raw_data = serde_json::json!({
            "val_id": resp.val_id,
            "transaction_id": resp.transaction_id,
            "status": resp.status,
            "amount": resp.amount,
            "currency": resp.currency,
            "card_type": resp.card_type,
            "bank_transaction_id": resp.bank_transaction_id,
        });

        Ok(GatewayValidationResponse {
            is_valid: resp.status == "VALID" || resp.status == "VALIDATED",
            amount: resp.amount,
            currency: resp.currency,
            transaction_id: resp.transaction_id,
            raw_data,
        })
    }

    fn verify_ipn_signature(&self, data: &HashMap<String, String>) -> bool {
        let ipn_data = SslcIpnData::from_hashmap(data);
        self.compute_ipn_signature(&ipn_data)
    }
}

impl SslcIpnData {
    fn from_hashmap(data: &HashMap<String, String>) -> Self {
        Self {
            val_id: data.get("val_id").cloned(),
            transaction_id: data
                .get("tran_id")
                .cloned()
                .or_else(|| data.get("transaction_id").cloned()),
            status: data.get("status").cloned(),
            amount: data.get("amount").cloned(),
            currency: data.get("currency").cloned(),
            card_issuer_country: data.get("card_issuer_country").cloned(),
            store_amount: data.get("store_amount").cloned(),
            verify_sign: data.get("verify_sign").cloned(),
            verify_key: data.get("verify_key").cloned(),
            extra: data
                .iter()
                .filter(|(k, _)| {
                    !matches!(
                        k.as_str(),
                        "val_id"
                            | "tran_id"
                            | "transaction_id"
                            | "status"
                            | "amount"
                            | "currency"
                            | "card_issuer_country"
                            | "store_amount"
                            | "verify_sign"
                            | "verify_key"
                    )
                })
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        }
    }
}

/// Errors returned by the SSLCommerz client.
#[derive(Debug, thiserror::Error)]
pub enum SslCommerzError {
    /// The HTTP request to the gateway failed.
    #[error("SSLCommerz request failed: {0}")]
    RequestFailed(String),
    /// Failed to parse the gateway response.
    #[error("SSLCommerz response parse failed: {0}")]
    ResponseParseFailed(String),
    /// The gateway returned a non-success status.
    #[error("SSLCommerz returned non-success: {status} - {reason}")]
    GatewayError {
        /// Status string from the gateway.
        status: String,
        /// Reason for failure.
        reason: String,
    },
}
