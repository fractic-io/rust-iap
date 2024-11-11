use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use fractic_server_error::{cxt, GenericServerError};

use crate::{
    data::{
        datasources::utils::validate_google_signature,
        models::google_cloud_rtdn_notifications::{
            developer_notification_model::DeveloperNotificationModel, pub_sub_model::PubSubModel,
        },
    },
    errors::GoogleCloudRtdnNotificationParseError,
};

#[async_trait]
pub(crate) trait GoogleCloudRtdnNotificationDatasource: Send + Sync {
    /// Parse Google Cloud RTDN Notification:
    /// https://developer.android.com/google/play/billing/rtdn-reference
    ///
    /// body:
    ///   The raw POST body of the notification.
    async fn parse_notification(
        &self,
        authorization_header: &str,
        body: &str,
    ) -> Result<(PubSubModel, DeveloperNotificationModel), GenericServerError>;
}

pub(crate) struct GoogleCloudRtdnNotificationDatasourceImpl {
    expected_aud: String,
}

#[async_trait]
impl GoogleCloudRtdnNotificationDatasource for GoogleCloudRtdnNotificationDatasourceImpl {
    async fn parse_notification(
        &self,
        authorization_header: &str,
        body: &str,
    ) -> Result<(PubSubModel, DeveloperNotificationModel), GenericServerError> {
        cxt!("GoogleCloudRtdnNotificationDatasourceImpl::parse_notification");
        validate_google_signature(authorization_header, &self.expected_aud).await?;
        let wrapper: PubSubModel = serde_json::from_str(body).map_err(|e| {
            GoogleCloudRtdnNotificationParseError::with_debug(
                CXT,
                "Failed to parse Pub/Sub wrapper.",
                format!("{:?}", e),
            )
        })?;
        let decoded_message = BASE64_STANDARD
            .decode(wrapper.message.data.clone())
            .map_err(|e| {
                GoogleCloudRtdnNotificationParseError::with_debug(
                    CXT,
                    "Failed to base64-decode notification struct.",
                    format!("{:?}", e),
                )
            })?;
        Ok((
            wrapper,
            serde_json::from_slice(&decoded_message).map_err(|e| {
                GoogleCloudRtdnNotificationParseError::with_debug(
                    CXT,
                    "Failed to parse notification struct.",
                    format!("{:?}", e),
                )
            })?,
        ))
    }
}

impl GoogleCloudRtdnNotificationDatasourceImpl {
    pub(crate) fn new(expected_aud: String) -> Self {
        Self { expected_aud }
    }
}
