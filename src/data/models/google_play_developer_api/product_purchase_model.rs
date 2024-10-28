#![allow(dead_code)]

use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

/// Data structure returned by the Google Play Developer API when querying for a
/// product purchase.
///
/// https://developers.google.com/android-publisher/api-ref/rest/v3/purchases.products#ProductPurchase
///
/// Whether fields are nullable is not documented explicitly in the API
/// reference, so reasonable assumptions are made.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductPurchaseModel {
    /// This kind represents an inappPurchase object in the androidpublisher
    /// service.
    pub(crate) kind: Option<String>,
    /// The time the product was purchased, in milliseconds since the epoch (Jan
    /// 1, 1970).
    #[serde(with = "ts_milliseconds")]
    pub(crate) purchase_time_millis: DateTime<Utc>,
    /// The purchase state of the order.
    pub(crate) purchase_state: PurchaseState,
    /// The consumption state of the inapp product.
    pub(crate) consumption_state: ConsumptionState,
    /// A developer-specified string that contains supplemental information
    /// about an order.
    pub(crate) developer_payload: Option<String>,
    /// The order id associated with the purchase of the inapp product.
    pub(crate) order_id: Option<String>,
    /// TThe type of purchase of the inapp product. This field is only set if
    /// this purchase was not made using the standard in-app billing flow.
    pub(crate) purchase_type: Option<PurchaseType>,
    /// The acknowledgement state of the inapp product.
    pub(crate) acknowledgement_state: AcknowledgementState,
    /// The purchase token generated to identify this purchase. May not be
    /// present.
    pub(crate) purchase_token: Option<String>,
    /// The inapp product SKU. May not be present.
    pub(crate) product_id: Option<String>,
    /// The quantity associated with the purchase of the inapp product. If not
    /// present, the quantity is 1.
    pub(crate) quantity: Option<i32>,
    /// An obfuscated version of the id that is uniquely associated with the
    /// user's account in your app. Only present if specified using
    /// https://developer.android.com/reference/com/android/billingclient/api/BillingFlowParams.Builder#setobfuscatedaccountid
    /// when the purchase was made.
    pub(crate) obfuscated_external_account_id: Option<String>,
    /// An obfuscated version of the id that is uniquely associated with the
    /// user's profile in your app. Only present if specified using
    /// https://developer.android.com/reference/com/android/billingclient/api/BillingFlowParams.Builder#setobfuscatedprofileid
    /// when the purchase was made.
    pub(crate) obfuscated_external_profile_id: Option<String>,
    /// ISO 3166-1 alpha-2 billing region code of the user at the time the
    /// product was granted.
    pub(crate) region_code: String,
    /// The quantity eligible for refund, i.e. quantity that hasn't been
    /// refunded. The value reflects quantity-based partial refunds and full
    /// refunds.
    pub(crate) refundable_quantity: Option<i32>,
}

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub(crate) enum PurchaseState {
    Purchased = 0,
    Canceled = 1,
    Pending = 2,
}

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub(crate) enum ConsumptionState {
    YetToBeConsumed = 0,
    Consumed = 1,
}

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub(crate) enum PurchaseType {
    Test = 0,
    Promo = 1,
    Rewarded = 2,
}

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub(crate) enum AcknowledgementState {
    YetToBeAcknowledged = 0,
    Acknowledged = 1,
}
