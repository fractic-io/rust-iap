use async_trait::async_trait;
use fractic_server_error::ServerError;
use reqwest::header::AUTHORIZATION;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    data::{
        datasources::utils::validate_and_parse_apple_jws,
        models::app_store_server_api::{
            jws_transaction_decoded_payload_model::JwsTransactionDecodedPayloadModel,
            send_test_notification_response::SendTestNotificationResponse,
            transaction_info_response_model::TransactionInfoResponseModel,
        },
    },
    errors::{AppStoreServerApiError, AppStoreServerApiKeyInvalid},
};

#[derive(Debug, Clone, Copy)]
enum Method {
    Post,
    Get,
}

#[async_trait]
pub(crate) trait AppStoreServerApiDatasource: Send + Sync {
    /// Get Transaction Info:
    /// https://developer.apple.com/documentation/appstoreserverapi/get_transaction_info
    ///
    /// transactionId:
    ///   The identifier of a transaction that belongs to the customer, and
    ///   which may be an original transaction identifier.
    async fn get_transaction_info(
        &self,
        transaction_id: &str,
    ) -> Result<JwsTransactionDecodedPayloadModel, ServerError>;

    /// Request a test notification from Apple.
    /// https://developer.apple.com/documentation/appstoreserverapi/request_a_test_notification
    async fn request_test_notification(&self, sandbox: bool) -> Result<String, ServerError>;
}

pub(crate) struct AppStoreServerApiDatasourceImpl {
    jwt_token: String,
    expected_aud: String,
}

#[async_trait]
impl AppStoreServerApiDatasource for AppStoreServerApiDatasourceImpl {
    async fn get_transaction_info(
        &self,
        transaction_id: &str,
    ) -> Result<JwsTransactionDecodedPayloadModel, ServerError> {
        let production_url = format!(
            "https://api.storekit.itunes.apple.com/inApps/v1/transactions/{transaction_id}"
        );
        let sandbox_url = format!(
            "https://api.storekit-sandbox.itunes.apple.com/inApps/v1/transactions/{transaction_id}"
        );
        let response_wrapper: TransactionInfoResponseModel = self
            .callout_with_sandbox_fallback(
                &production_url,
                &sandbox_url,
                "GetTransactionInfo",
                Method::Get,
            )
            .await?;
        validate_and_parse_apple_jws(
            &response_wrapper.signed_transaction_info,
            &self.expected_aud,
        )
        .await
    }

    async fn request_test_notification(&self, sandbox: bool) -> Result<String, ServerError> {
        let url = match sandbox {
            false => "https://api.storekit.itunes.apple.com/inApps/v1/notifications/test",
            true => "https://api.storekit-sandbox.itunes.apple.com/inApps/v1/notifications/test",
        };
        Ok(self
            .callout::<SendTestNotificationResponse>(url, "RequestTestNotification", Method::Post)
            .await?
            .test_notification_token)
    }
}

impl AppStoreServerApiDatasourceImpl {
    pub(crate) async fn new(
        api_key: &str,
        key_id: &str,
        issuer_id: &str,
        bundle_id: &str,
        expected_aud: String,
    ) -> Result<Self, ServerError> {
        Ok(Self {
            jwt_token: Self::build_jwt_token(api_key, key_id, issuer_id, bundle_id).await?,
            expected_aud,
        })
    }

    async fn build_jwt_token(
        api_key: &str,
        key_id: &str,
        issuer_id: &str,
        bundle_id: &str,
    ) -> Result<String, ServerError> {
        // Build header.
        let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256);
        header.kid = Some(key_id.to_owned());

        // Build claims.
        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            iss: String,
            iat: usize,
            exp: usize,
            aud: String,
            bid: String,
        }
        let claims = Claims {
            iss: issuer_id.to_owned(),
            iat: chrono::Utc::now().timestamp() as usize,
            exp: (chrono::Utc::now() + chrono::Duration::minutes(10)).timestamp() as usize,
            aud: "appstoreconnect-v1".to_owned(),
            bid: bundle_id.to_owned(),
        };

        // Build token.
        jsonwebtoken::encode(
            &header,
            &claims,
            &jsonwebtoken::EncodingKey::from_ec_pem(api_key.as_ref())
                .map_err(|e| AppStoreServerApiKeyInvalid::with_debug("invalid key format", &e))?,
        )
        .map_err(|e| AppStoreServerApiKeyInvalid::with_debug("failed to build JWT token", &e))
    }

    async fn callout_with_sandbox_fallback<T: DeserializeOwned>(
        &self,
        production_url: &str,
        sandbox_url: &str,
        function_name: &str,
        method: Method,
    ) -> Result<T, ServerError> {
        // As per Apple's documentation, try production endpoint first. If it
        // fails, try checking the sandbox.
        //
        // If both fail, we will return the error from the production callout.
        match self.callout(production_url, function_name, method).await {
            Ok(production_response) => Ok(production_response),
            Err(production_error) => match self.callout(sandbox_url, function_name, method).await {
                Ok(sandbox_response) => Ok(sandbox_response),
                Err(_sandbox_error) => Err(production_error),
            },
        }
    }

    async fn callout<T: DeserializeOwned>(
        &self,
        url: &str,
        function_name: &str,
        method: Method,
    ) -> Result<T, ServerError> {
        let client = reqwest::Client::new();
        let builder = match method {
            Method::Post => client.post(url),
            Method::Get => client.get(url),
        };
        let response = builder
            .header(AUTHORIZATION, format!("Bearer {}", self.jwt_token))
            .send()
            .await
            .map_err(|e| {
                AppStoreServerApiError::with_debug(function_name, "callout failed to send", &e)
            })?;

        if !response.status().is_success() {
            return Err(AppStoreServerApiError::with_debug(
                function_name,
                &format!(
                    "callout returned with {} status code",
                    response.status().to_string(),
                ),
                &response.text().await.unwrap_or_default(),
            ));
        }

        response.json().await.map_err(|e| {
            AppStoreServerApiError::with_debug(
                function_name,
                "failed to parse callout response",
                &e,
            )
        })
    }
}
