use async_trait::async_trait;
use fractic_generic_server_error::{cxt, GenericServerError};

use crate::{
    data::{
        datasources::utils::{decode_jws_payload, validate_apple_signature},
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
        GenericServerError,
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
        GenericServerError,
    > {
        cxt!("AppStoreServerNotificationDatasourceImpl::parse_notification");
        let wrapper: ResponseBodyV2Model = serde_json::from_str(body).map_err(|e| {
            AppStoreServerNotificationParseError::with_debug(
                CXT,
                "Failed to parse notification",
                format!("{:?}", e),
            )
        })?;
        validate_apple_signature(&wrapper.signed_payload, &self.expected_aud).await?;
        let decoded_payload: ResponseBodyV2DecodedPayloadModel =
            decode_jws_payload(CXT, &wrapper.signed_payload)?;
        let decoded_transaction_info: Option<JwsTransactionDecodedPayloadModel> = decoded_payload
            .data
            .as_ref()
            .map(|data| decode_jws_payload(CXT, &data.signed_transaction_info))
            .transpose()?;
        let decoded_renewal_info: Option<JwsRenewalInfoDecodedPayloadModel> = decoded_payload
            .data
            .as_ref()
            .map(|data| data.signed_renewal_info.as_ref())
            .flatten()
            .map(|renewal_info| decode_jws_payload(CXT, &renewal_info))
            .transpose()?;
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
