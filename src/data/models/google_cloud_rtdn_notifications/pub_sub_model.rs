#![allow(dead_code)]

use std::collections::HashMap;

use serde::Deserialize;

/// Data structure returned for all Google Cloud Pub/Sub topic notifications.
///
/// https://developer.android.com/google/play/billing/rtdn-reference
///
/// Whether fields are nullable is not documented explicitly in the API
/// reference, so reasonable assumptions are made.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PubSubModel {
    pub(crate) message: Message,
    pub(crate) subscription: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Message {
    #[serde(default)]
    pub(crate) attributes: HashMap<String, String>,
    /// Main data. Base64-encoded JSON object.
    pub(crate) data: String,
    pub(crate) message_id: String,
}
