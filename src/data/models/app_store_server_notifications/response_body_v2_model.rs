#![allow(dead_code)]

use serde::Deserialize;

type SignedPayload = String;

/// Data structure sent by the App Store Server Notifications.
///
/// https://developer.apple.com/documentation/appstoreservernotifications/responsebodyv2
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResponseBodyV2Model {
    /// The payload in JSON Web Signature (JWS) format, signed by the App Store.
    pub(crate) signed_payload: SignedPayload,
}
