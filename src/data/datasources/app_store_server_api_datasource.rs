use async_trait::async_trait;
use fractic_generic_server_error::{cxt, GenericServerError};
use reqwest::header::AUTHORIZATION;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    data::{
        datasources::utils::decode_jws_payload,
        models::app_store_server_api::{
            jws_transaction_decoded_payload_model::JwsTransactionDecodedPayloadModel,
            transaction_info_response_model::TransactionInfoResponseModel,
        },
    },
    errors::{AppStoreServerApiError, AppStoreServerApiKeyInvalid},
};

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
    ) -> Result<JwsTransactionDecodedPayloadModel, GenericServerError>;
}

pub(crate) struct AppStoreServerApiDatasourceImpl {
    jwt_token: String,
}

#[async_trait]
impl AppStoreServerApiDatasource for AppStoreServerApiDatasourceImpl {
    async fn get_transaction_info(
        &self,
        transaction_id: &str,
    ) -> Result<JwsTransactionDecodedPayloadModel, GenericServerError> {
        cxt!("AppStoreServerApiDatasourceImpl::get_transaction_info");
        let production_url = format!(
            "https://api.storekit.itunes.apple.com/inApps/v1/transactions/{transaction_id}"
        );
        let sandbox_url = format!(
            "https://api.storekit-sandbox.itunes.apple.com/inApps/v1/transactions/{transaction_id}"
        );
        let response_wrapper: TransactionInfoResponseModel = self
            .callout_with_sandbox_fallback(CXT, &production_url, &sandbox_url, "GetTransactionInfo")
            .await?;
        decode_jws_payload(CXT, &response_wrapper.signed_transaction_info)
    }
}

impl AppStoreServerApiDatasourceImpl {
    pub(crate) async fn new(
        api_key: &str,
        key_id: &str,
        issuer_id: &str,
        bundle_id: &str,
    ) -> Result<Self, GenericServerError> {
        Ok(Self {
            jwt_token: Self::build_jwt_token(api_key, key_id, issuer_id, bundle_id).await?,
        })
    }

    async fn build_jwt_token(
        api_key: &str,
        key_id: &str,
        issuer_id: &str,
        bundle_id: &str,
    ) -> Result<String, GenericServerError> {
        cxt!("AppStoreServerApiDatasourceImpl::build_jwt_token");

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
            &jsonwebtoken::EncodingKey::from_secret(api_key.as_ref()),
        )
        .map_err(|e| {
            AppStoreServerApiKeyInvalid::with_debug(
                CXT,
                "Failed to build JWT token.",
                format!("{:?}", e),
            )
        })
    }

    async fn callout_with_sandbox_fallback<T: DeserializeOwned>(
        &self,
        cxt: &'static str,
        production_url: &str,
        sandbox_url: &str,
        function_name: &str,
    ) -> Result<T, GenericServerError> {
        // As per Apple's documentation, try production endpoint first. If it
        // fails, try checking the sandbox.
        //
        // If both fail, we will return the error from the production callout.
        match self.callout(cxt, production_url, function_name).await {
            Ok(production_response) => Ok(production_response),
            Err(production_error) => match self.callout(cxt, sandbox_url, function_name).await {
                Ok(sandbox_response) => Ok(sandbox_response),
                Err(_sandbox_error) => Err(production_error),
            },
        }
    }

    async fn callout<T: DeserializeOwned>(
        &self,
        cxt: &'static str,
        url: &str,
        function_name: &str,
    ) -> Result<T, GenericServerError> {
        let response = reqwest::Client::new()
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.jwt_token))
            .send()
            .await
            .map_err(|e| {
                AppStoreServerApiError::with_debug(
                    cxt,
                    "Callout failed to send.",
                    format!("{}; {:?}", function_name, e),
                )
            })?;

        if !response.status().is_success() {
            return Err(AppStoreServerApiError::with_debug(
                cxt,
                "Callout returned with non-200 status code.",
                format!(
                    "{}; {}; {}",
                    function_name,
                    response.status().to_string(),
                    response.text().await.unwrap_or_default()
                ),
            ));
        }

        response.json().await.map_err(|e| {
            AppStoreServerApiError::with_debug(
                cxt,
                "Failed to parse callout response.",
                format!("{}; {:?}", function_name, e),
            )
        })
    }
}
