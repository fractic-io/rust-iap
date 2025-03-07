#![allow(dead_code)]

use serde::Deserialize;

type JWSTransaction = String;

/// Data structure returned by the App Store Server API when requesting a test
/// S2S notification.
///
/// https://developer.apple.com/documentation/appstoreserverapi/sendtestnotificationresponse
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SendTestNotificationResponse {
    /// The test notification token that uniquely identifies the notification
    /// test that App Store Server Notifications sends to your server.
    pub(crate) test_notification_token: String,
}
