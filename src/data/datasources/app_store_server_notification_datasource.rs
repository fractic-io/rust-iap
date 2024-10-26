use fractic_generic_server_error::{cxt, GenericServerError};

use crate::{
    data::{
        datasources::utils::decode_jws_payload,
        models::{
            app_store_server_api::{
                jws_renewal_info_decoded_payload_model::JWSRenewalInfoDecodedPayloadModel,
                jws_transaction_decoded_payload_model::JWSTransactionDecodedPayloadModel,
            },
            app_store_server_notifications::{
                response_body_v2_decoded_payload_model::ResponseBodyV2DecodedPayloadModel,
                response_body_v2_model::ResponseBodyV2Model,
            },
        },
    },
    errors::AppStoreServerNotificationParseError,
};

pub(crate) trait AppStoreServerNotificationDatasource {
    /// Parse App Store Server Notification:
    /// https://developer.apple.com/documentation/appstoreservernotifications/app-store-server-notifications-v2
    ///
    /// notification:
    ///   The raw POST body of the notification.
    async fn parse_notification(
        &self,
        notification: &str,
    ) -> Result<
        (
            ResponseBodyV2DecodedPayloadModel,
            Option<JWSTransactionDecodedPayloadModel>,
            Option<JWSRenewalInfoDecodedPayloadModel>,
        ),
        GenericServerError,
    >;
}

pub(crate) struct AppStoreServerNotificationDatasourceImpl;

impl AppStoreServerNotificationDatasource for AppStoreServerNotificationDatasourceImpl {
    async fn parse_notification(
        &self,
        notification: &str,
    ) -> Result<
        (
            ResponseBodyV2DecodedPayloadModel,
            Option<JWSTransactionDecodedPayloadModel>,
            Option<JWSRenewalInfoDecodedPayloadModel>,
        ),
        GenericServerError,
    > {
        cxt!("AppStoreServerNotificationDatasourceImpl::parse_notification");
        let wrapper: ResponseBodyV2Model = serde_json::from_str(notification).map_err(|e| {
            AppStoreServerNotificationParseError::with_debug(
                CXT,
                "Failed to parse notification",
                format!("{:?}", e),
            )
        })?;
        let decoded_payload: ResponseBodyV2DecodedPayloadModel =
            decode_jws_payload(CXT, &wrapper.signed_payload)?;
        let decoded_transaction_info: Option<JWSTransactionDecodedPayloadModel> = decoded_payload
            .data
            .as_ref()
            .map(|data| decode_jws_payload(CXT, &data.signed_transaction_info))
            .transpose()?;
        let decoded_renewal_info: Option<JWSRenewalInfoDecodedPayloadModel> = decoded_payload
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
    pub(crate) fn new() -> Self {
        Self
    }
}
