#![allow(dead_code)]

use serde::Deserialize;
use serde_repr::Deserialize_repr;

/// Data structure for Google Play Real-time Developer Notifications (RTDN).
///
/// https://developer.android.com/google/play/billing/rtdn-reference
///
/// Whether fields are nullable is not documented explicitly in the API
/// reference, so reasonable assumptions are made.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeveloperNotificationModel {
    /// The version of this notification. Initially, this is "1.0". This version
    /// is distinct from other version fields.
    pub(crate) version: String,
    /// The package name of the application that this notification relates to
    /// (for example, `com.some.thing`).
    pub(crate) package_name: String,
    /// The timestamp when the event occurred, in milliseconds since the Epoch.
    pub(crate) event_time_millis: i64,
    /// If this field is present, then this notification is related to a
    /// subscription, and this field contains additional information related to
    /// the subscription. Note that this field is mutually exclusive with
    /// oneTimeProductNotification, voidedPurchaseNotification, and
    /// testNotification.
    pub(crate) subscription_notification: Option<SubscriptionNotification>,
    /// If this field is present, then this notification is related to a
    /// one-time purchase, and this field contains additional information
    /// related to the purchase. Note that this field is mutually exclusive with
    /// subscriptionNotification, voidedPurchaseNotification, and
    /// testNotification.
    pub(crate) one_time_product_notification: Option<OneTimeProductNotification>,
    /// If this field is present, then this notification is related to a voided
    /// purchase, and this field contains additional information related to the
    /// voided purchase. Note that this field is mutually exclusive with
    /// oneTimeProductNotification, subscriptionNotification, and
    /// testNotification.
    pub(crate) voided_purchase_notification: Option<VoidedPurchaseNotification>,
    /// If this field is present, then this notification is related to a test
    /// publish. These are sent only through the Google Play Developer Console.
    /// Note that this field is mutually exclusive with
    /// oneTimeProductNotification, subscriptionNotification, and
    /// voidedPurchaseNotification.
    pub(crate) test_notification: Option<TestNotification>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionNotification {
    /// The version of this notification. Initially, this is "1.0". This version
    /// is distinct from other version fields.
    pub(crate) version: String,
    /// The type of notification.
    pub(crate) notification_type: SubscriptionNotificationType,
    /// The token provided to the user's device when the subscription was
    /// purchased.
    pub(crate) purchase_token: String,
    /// The purchased subscription's product ID (for example, "monthly001").
    pub(crate) subscription_id: String,
}

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub(crate) enum SubscriptionNotificationType {
    /// A subscription was recovered from account hold.
    SubscriptionRecovered = 1,
    /// An active subscription was renewed.
    SubscriptionRenewed = 2,
    /// A subscription was either voluntarily or involuntarily cancelled. For
    /// voluntary cancellation, sent when the user cancels.
    SubscriptionCanceled = 3,
    /// A new subscription was purchased.
    SubscriptionPurchased = 4,
    /// A subscription has entered account hold (if enabled).
    SubscriptionOnHold = 5,
    /// A subscription has entered grace period (if enabled).
    SubscriptionInGracePeriod = 6,
    /// User has restored their subscription from Play > Account >
    /// Subscriptions. The subscription was canceled but had not expired yet
    /// when the user restores. For more information, see Restorations.
    SubscriptionRestarted = 7,
    /// A subscription price change has successfully been confirmed by the user.
    SubscriptionPriceChangeConfirmed = 8,
    /// A subscription's recurrence time has been extended.
    SubscriptionDeferred = 9,
    /// A subscription has been paused.
    SubscriptionPaused = 10,
    /// A subscription pause schedule has been changed.
    SubscriptionPauseScheduleChanged = 11,
    /// A subscription has been revoked from the user before the expiration
    /// time.
    SubscriptionRevoked = 12,
    /// A subscription has expired.
    SubscriptionExpired = 13,
    /// A pending transaction of a subscription has been canceled.
    SubscriptionPendingPurchaseCanceled = 20,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OneTimeProductNotification {
    /// The version of this notification. Initially, this will be "1.0". This
    /// version is distinct from other version fields.
    pub(crate) version: String,
    /// The type of notification.
    pub(crate) notification_type: OneTimeProductNotificationType,
    /// The token provided to the user’s device when purchase was made.
    pub(crate) purchase_token: String,
    /// The purchased one-time product ID (for example, "sword_001")
    pub(crate) sku: String,
}

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub(crate) enum OneTimeProductNotificationType {
    /// A one-time product was successfully purchased by a user.
    OneTimeProductPurchased = 1,
    /// A pending one-time product purchase has been canceled by the user.
    OneTimeProductCanceled = 2,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VoidedPurchaseNotification {
    /// The token associated with the purchase that has been voided. This
    /// information is provided to the developer when a new purchase occurs.
    pub(crate) purchase_token: String,
    /// The unique order ID associated with the transaction that has been
    /// voided. For one time purchases, this represents the only order ID
    /// generated for the purchase. For auto-renewing subscriptions, a new order
    /// ID is generated for each renewal transaction.
    pub(crate) order_id: String,
    /// The product type for a voided purchase.
    pub(crate) product_type: VoidedPurchaseProductType,
    /// The refund type for a voided purchase.
    ///
    /// Note when the remaining total quantity of a multi-quantity purchase is
    /// refunded, the refundType will be REFUND_TYPE_FULL_REFUND.
    pub(crate) refund_type: VoidedPurchaseRefundType,
}

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub(crate) enum VoidedPurchaseProductType {
    /// A subscription purchase has been voided.
    ProductTypeSubscription = 1,
    /// A one-time purchase has been voided.
    ProductTypeOneTime = 2,
}

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub(crate) enum VoidedPurchaseRefundType {
    /// The purchase has been fully voided.
    RefundTypeFullRefund = 1,
    /// The purchase has been partially voided by a quantity-based partial
    /// refund, applicable only to multi-quantity purchases. A purchase can be
    /// partially voided multiple times.
    RefundTypeQuantityBasedPartialRefund = 2,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TestNotification {
    /// The version of this notification. Initially, this is "1.0". This version
    /// is distinct from other version fields.
    pub(crate) version: String,
}
