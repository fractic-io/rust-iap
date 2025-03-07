#![allow(dead_code)]

use chrono::{
    serde::{ts_milliseconds, ts_milliseconds_option},
    DateTime, Utc,
};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

use super::common::{Environment, OfferDiscountType, OfferType};

/// Data structure for the decoded payload of a JWSRenewalInfo, returned by the
/// App Store Server API.
///
/// https://developer.apple.com/documentation/appstoreserverapi/jwsrenewalinfodecodedpayload
///
/// Whether fields are nullable is not documented explicitly in the API
/// reference, so reasonable assumptions are made.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JwsRenewalInfoDecodedPayloadModel {
    /// The identifier of the product that renews at the next billing period.
    pub(crate) auto_renew_product_id: String,
    /// The renewal status of the auto-renewable subscription.
    pub(crate) auto_renew_status: AutoRenewStatus,
    /// The currency code for the renewalPrice of the subscription.
    pub(crate) currency: Option<String>,
    /// The list of win-back offer IDs that the customer is eligible for.
    #[serde(default)]
    pub(crate) eligible_win_back_offer_ids: Vec<String>,
    /// The server environment, either sandbox or production.
    pub(crate) environment: Environment,
    /// The reason the subscription expired.
    pub(crate) expiration_intent: Option<ExpirationIntent>,
    /// The time when the Billing Grace Period for subscription renewals
    /// expires.
    #[serde(default, with = "ts_milliseconds_option")]
    pub(crate) grace_period_expires_date: Option<DateTime<Utc>>,
    /// A Boolean value that indicates whether the App Store is attempting to
    /// automatically renew the expired subscription.
    #[serde(default)]
    pub(crate) is_in_billing_retry_period: bool,
    /// The payment mode of the discount offer.
    pub(crate) offer_discount_type: Option<OfferDiscountType>,
    /// The offer code or the promotional offer identifier.
    pub(crate) offer_identifier: Option<String>,
    /// The type of subscription offer.
    pub(crate) offer_type: Option<OfferType>,
    /// The transaction identifier of the original purchase associated with this
    /// transaction.
    pub(crate) original_transaction_id: Option<String>,
    /// The status that indicates whether the auto-renewable subscription is
    /// subject to a price increase.
    pub(crate) price_increase_status: Option<PriceIncreaseStatus>,
    /// The product identifier of the In-App Purchase.
    pub(crate) product_id: String,
    /// The earliest start date of the auto-renewable subscription in a series
    /// of subscription purchases that ignores all lapses of paid service that
    /// are 60 days or fewer.
    #[serde(default, with = "ts_milliseconds_option")]
    pub(crate) recent_subscription_start_date: Option<DateTime<Utc>>,
    /// The UNIX time, in milliseconds, when the most recent auto-renewable
    /// subscription purchase expires.
    #[serde(default, with = "ts_milliseconds_option")]
    pub(crate) renewal_date: Option<DateTime<Utc>>,
    /// The renewal price, in milliunits, of the auto-renewable subscription
    /// that renews at the next billing period.
    pub(crate) renewal_price: Option<i64>,
    /// The UNIX time, in milliseconds, that the App Store signed the JSON Web
    /// Signature (JWS) data.
    #[serde(with = "ts_milliseconds")]
    pub(crate) signed_date: DateTime<Utc>,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
pub(crate) enum AutoRenewStatus {
    /// Automatic renewal is off. The customer has turned off automatic renewal
    /// for the subscription, and it won’t renew at the end of the current
    /// subscription period.
    Off = 0,
    /// Automatic renewal is on. The subscription renews at the end of the
    /// current subscription period.
    On = 1,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
pub(crate) enum ExpirationIntent {
    /// The customer canceled their subscription.
    VoluntaryCancellation = 1,
    /// Billing error; for example, the customer’s payment information is no
    /// longer valid.
    BillingError = 2,
    /// The customer didn’t consent to an auto-renewable subscription price
    /// increase that requires customer consent, allowing the subscription to
    /// expire.
    PriceIncreaseDecline = 3,
    /// The product wasn’t available for purchase at the time of renewal.
    ProductUnavailable = 4,
    /// The subscription expired for some other reason.
    Other = 5,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
pub(crate) enum PriceIncreaseStatus {
    /// The customer hasn’t yet responded to an auto-renewable subscription
    /// price increase that requires customer consent.
    NoActionTaken = 0,
    /// The customer consented to an auto-renewable subscription price increase
    /// that requires customer consent, or the App Store has notified the
    /// customer of an auto-renewable subscription price increase that doesn’t
    /// require consent.
    CustomerConsented = 1,
}
