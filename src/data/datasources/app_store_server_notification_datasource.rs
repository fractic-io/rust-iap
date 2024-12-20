use async_trait::async_trait;
use fractic_server_error::ServerError;

use crate::{
    data::{
        datasources::utils::validate_and_parse_apple_jws,
        models::{
            app_store_server_api::{
                jws_renewal_info_decoded_payload_model::JwsRenewalInfoDecodedPayloadModel,
                jws_transaction_decoded_payload_model::JwsTransactionDecodedPayloadModel,
            },
            app_store_server_notifications::{
                response_body_v2_decoded_payload_model::ResponseBodyV2DecodedPayloadModel,
                response_body_v2_model::ResponseBodyV2Model,
            },
        },
    },
    errors::AppStoreServerNotificationParseError,
};

#[async_trait]
pub(crate) trait AppStoreServerNotificationDatasource: Send + Sync {
    /// Parse App Store Server Notification:
    /// https://developer.apple.com/documentation/appstoreservernotifications/app-store-server-notifications-v2
    ///
    /// body:
    ///   The raw POST body of the notification.
    async fn parse_notification(
        &self,
        body: &str,
    ) -> Result<
        (
            ResponseBodyV2DecodedPayloadModel,
            Option<JwsTransactionDecodedPayloadModel>,
            Option<JwsRenewalInfoDecodedPayloadModel>,
        ),
        ServerError,
    >;
}

pub(crate) struct AppStoreServerNotificationDatasourceImpl {
    expected_aud: String,
}

#[async_trait]
impl AppStoreServerNotificationDatasource for AppStoreServerNotificationDatasourceImpl {
    async fn parse_notification(
        &self,
        body: &str,
    ) -> Result<
        (
            ResponseBodyV2DecodedPayloadModel,
            Option<JwsTransactionDecodedPayloadModel>,
            Option<JwsRenewalInfoDecodedPayloadModel>,
        ),
        ServerError,
    > {
        let wrapper: ResponseBodyV2Model = serde_json::from_str(body)
            .map_err(|e| AppStoreServerNotificationParseError::with_debug(&e))?;
        let decoded_payload: ResponseBodyV2DecodedPayloadModel =
            validate_and_parse_apple_jws(&wrapper.signed_payload, &self.expected_aud).await?;
        let decoded_transaction_info: Option<JwsTransactionDecodedPayloadModel> =
            match decoded_payload
                .data
                .as_ref()
                .map(|data| data.signed_transaction_info.as_ref())
                .flatten()
            {
                Some(transaction_info) => {
                    Some(validate_and_parse_apple_jws(transaction_info, &self.expected_aud).await?)
                }
                None => None,
            };
        let decoded_renewal_info: Option<JwsRenewalInfoDecodedPayloadModel> = match decoded_payload
            .data
            .as_ref()
            .map(|data| data.signed_renewal_info.as_ref())
            .flatten()
        {
            Some(renewal_info) => {
                Some(validate_and_parse_apple_jws(renewal_info, &self.expected_aud).await?)
            }
            None => None,
        };
        Ok((
            decoded_payload,
            decoded_transaction_info,
            decoded_renewal_info,
        ))
    }
}

impl AppStoreServerNotificationDatasourceImpl {
    pub(crate) fn new(expected_aud: String) -> Self {
        Self { expected_aud }
    }
}
