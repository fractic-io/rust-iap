#![allow(dead_code)]

use serde::Deserialize;
use serde_repr::Deserialize_repr;

use super::common::{Environment, OfferDiscountType, OfferType};

type TimestampType = u64;

/// Data structure for the decoded payload of a JWSTransaction, returned by the
/// App Store Server API.
///
/// https://developer.apple.com/documentation/appstoreserverapi/jwstransactiondecodedpayload
///
/// Whether fields are nullable is not documented explicitly in the API
/// reference, so reasonable assumptions are made.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JWSTransactionDecodedPayloadModel {
    /// A UUID you create at the time of purchase that associates the
    /// transaction with a customer on your own service. If your app doesn’t
    /// provide an appAccountToken, this string is empty. For more information,
    /// see appAccountToken(_:).
    pub(crate) app_account_token: Option<String>,
    /// The bundle identifier of the app.
    pub(crate) bundle_id: String,
    /// The three-letter ISO 4217 currency code associated with the price
    /// parameter. This value is present only if price is present.
    pub(crate) currency: Option<String>,
    /// The server environment, either sandbox or production.
    pub(crate) environment: Environment,
    /// The UNIX time, in milliseconds, that the subscription expires or renews.
    pub(crate) expires_date: Option<TimestampType>,
    /// A string that describes whether the transaction was purchased by the
    /// customer, or is available to them through Family Sharing.
    pub(crate) in_app_ownership_type: Option<InAppOwnershipType>,
    /// A Boolean value that indicates whether the customer upgraded to another
    /// subscription.
    #[serde(default)]
    pub(crate) is_upgraded: bool,
    /// The payment mode you configure for the subscription offer, such as Free
    /// Trial, Pay As You Go, or Pay Up Front.
    pub(crate) offer_discount_type: Option<OfferDiscountType>,
    /// The identifier that contains the offer code or the promotional offer
    /// identifier.
    pub(crate) offer_identifier: Option<String>,
    /// A value that represents the promotional offer type.
    pub(crate) offer_type: Option<OfferType>,
    /// The UNIX time, in milliseconds, that represents the purchase date of the
    /// original transaction identifier.
    pub(crate) original_purchase_date: Option<TimestampType>,
    /// The transaction identifier of the original purchase.
    pub(crate) original_transaction_id: String,
    /// An integer value that represents the price multiplied by 1000 of the
    /// in-app purchase or subscription offer you configured in App Store
    /// Connect and that the system records at the time of the purchase. For
    /// more information, see price. The currency parameter indicates the
    /// currency of this price.
    pub(crate) price: Option<i64>,
    /// The unique identifier of the product.
    pub(crate) product_id: String,
    /// The UNIX time, in milliseconds, that the App Store charged the
    /// customer’s account for a purchase, restored product, subscription, or
    /// subscription renewal after a lapse.
    pub(crate) purchase_date: TimestampType,
    /// The number of consumable products the customer purchased.
    pub(crate) quantity: i32,
    /// The UNIX time, in milliseconds, that the App Store refunded the
    /// transaction or revoked it from Family Sharing.
    pub(crate) revocation_date: Option<TimestampType>,
    /// The reason that the App Store refunded the transaction or revoked it
    /// from Family Sharing.
    pub(crate) revocation_reason: Option<RevocationReason>,
    /// The UNIX time, in milliseconds, that the App Store signed the JSON Web
    /// Signature (JWS) data.
    pub(crate) signed_date: TimestampType,
    /// The three-letter code that represents the country or region associated
    /// with the App Store storefront for the purchase.
    pub(crate) storefront: Option<String>,
    /// An Apple-defined value that uniquely identifies the App Store storefront
    /// associated with the purchase.
    pub(crate) storefront_id: Option<String>,
    /// The identifier of the subscription group to which the subscription
    /// belongs.
    pub(crate) subscription_group_identifier: Option<String>,
    /// The unique identifier of the transaction.
    pub(crate) transaction_id: String,
    /// The reason for the purchase transaction, which indicates whether it’s a
    /// customer’s purchase or a renewal for an auto-renewable subscription that
    /// the system initiates.
    pub(crate) transaction_reason: Option<TransactionReason>,
    /// The type of the in-app purchase.
    #[serde(rename = "type")]
    pub(crate) transaction_type: Option<TransactionType>,
    /// The unique identifier of subscription purchase events across devices,
    /// including subscription renewals.
    pub(crate) web_order_line_item_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum InAppOwnershipType {
    /// The transaction belongs to a family member who benefits from service.
    FamilyShared,
    /// The transaction belongs to the purchaser.
    Purchased,

    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
pub(crate) enum RevocationReason {
    /// The App Store refunded the transaction on behalf of the customer for
    /// other reasons, for example, an accidental purchase.
    Other = 0,
    /// The App Store refunded the transaction on behalf of the customer due to
    /// an actual or perceived issue within your app.
    Issue = 1,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum TransactionReason {
    /// The customer initiated the purchase, which may be for any in-app
    /// purchase type: consumable, non-consumable, non-renewing subscription, or
    /// auto-renewable subscription.
    Purchase,
    /// The App Store server initiated the purchase transaction to renew an
    /// auto-renewable subscription.
    Renewal,

    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize)]
pub(crate) enum TransactionType {
    /// An auto-renewable subscription.
    #[serde(rename = "Auto-Renewable Subscription")]
    AutoRenewableSubscription,
    /// A non-consumable In-App Purchase.
    #[serde(rename = "Non-Consumable")]
    NonConsumable,
    /// A consumable In-App Purchase.
    #[serde(rename = "Consumable")]
    Consumable,
    /// A non-renewing subscription.
    #[serde(rename = "Non-Renewing Subscription")]
    NonRenewableSubscription,

    #[serde(untagged)]
    Unknown(String),
}
