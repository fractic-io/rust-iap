#![allow(dead_code)]

use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::data::models::app_store_server_api::common::Environment;

type AppleIdType = u64;
type JWSTransaction = String;
type JWSRenewalInfo = String;

/// Data structure for the decoded payload of a SignedPayload, returned by the
/// App Store Server Notifications service.
///
/// https://developer.apple.com/documentation/appstoreservernotifications/responsebodyv2decodedpayload
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResponseBodyV2DecodedPayloadModel {
    /// The in-app purchase event for which the App Store sends this version 2
    /// notification.
    pub(crate) notification_type: NotificationType,
    /// Additional information that identifies the notification event. The
    /// subtype field is present only for specific version 2 notifications.
    pub(crate) subtype: Option<NotificationSubtype>,
    /// The object that contains the app metadata and signed renewal and
    /// transaction information. The data, summary, and externalPurchaseToken
    /// fields are mutually exclusive. The payload contains only one of these
    /// fields.
    pub(crate) data: Option<NotificationData>,
    /// The summary data that appears when the App Store server completes your
    /// request to extend a subscription renewal date for eligible subscribers.
    /// For more information, see Extend Subscription Renewal Dates for All
    /// Active Subscribers. The data, summary, and externalPurchaseToken fields
    /// are mutually exclusive. The payload contains only one of these fields.
    pub(crate) summary: Option<NotificationSummary>,
    /// This field appears when the notificationType is EXTERNAL_PURCHASE_TOKEN.
    /// The data, summary, and externalPurchaseToken fields are mutually
    /// exclusive. The payload contains only one of these fields.
    pub(crate) external_purchase_token: Option<ExternalPurchaseToken>,
    /// The App Store Server Notification version number, "2.0".
    pub(crate) version: String,
    /// The UNIX time, in milliseconds, that the App Store signed the JSON Web
    /// Signature data.
    #[serde(with = "ts_milliseconds")]
    pub(crate) signed_date: DateTime<Utc>,
    /// A unique identifier for the notification. Use this value to identify a
    /// duplicate notification.
    #[serde(rename = "notificationUUID")]
    pub(crate) notification_uuid: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum NotificationType {
    /// A notification type that, along with its subtype, indicates that the
    /// customer subscribed to an auto-renewable subscription. If the subtype is
    /// INITIAL_BUY, the customer either purchased or received access through
    /// Family Sharing to the subscription for the first time. If the subtype is
    /// RESUBSCRIBE, the user resubscribed or received access through Family
    /// Sharing to the same subscription or to another subscription within the
    /// same subscription group.
    ///
    /// For notifications about other product type purchases, see the
    /// ONE_TIME_CHARGE notification type.
    Subscribed,
    /// A notification type that, along with its subtype, indicates that the
    /// customer made a change to their subscription plan. If the subtype is
    /// UPGRADE, the user upgraded their subscription. The upgrade goes into
    /// effect immediately, starting a new billing period, and the user receives
    /// a prorated refund for the unused portion of the previous period. If the
    /// subtype is DOWNGRADE, the customer downgraded their subscription.
    /// Downgrades take effect at the next renewal date and don’t affect the
    /// currently active plan.
    ///
    /// If the subtype is empty, the user changed their renewal preference back
    /// to the current subscription, effectively canceling a downgrade.
    ///
    /// For more information on subscription levels, see Ranking subscriptions
    /// within the group.
    DidChangeRenewalPref,
    /// A notification type that, along with its subtype, indicates that the
    /// customer made a change to the subscription renewal status. If the
    /// subtype is AUTO_RENEW_ENABLED, the customer reenabled subscription
    /// auto-renewal. If the subtype is AUTO_RENEW_DISABLED, the customer
    /// disabled subscription auto-renewal, or the App Store disabled
    /// subscription auto-renewal after the customer requested a refund.
    DidChangeRenewalStatus,
    /// A notification type that indicates that a customer with an active
    /// subscription redeemed a subscription offer.
    ///
    /// If the subtype is UPGRADE, the customer redeemed an offer to upgrade
    /// their active subscription, which goes into effect immediately. If the
    /// subtype is DOWNGRADE, the customer redeemed an offer to downgrade their
    /// active subscription, which goes into effect at the next renewal date. If
    /// the customer redeemed an offer for their active subscription, you
    /// receive an OFFER_REDEEMED notification type without a subtype.
    ///
    /// For more information about subscription offer codes, see Supporting
    /// subscription offer codes in your app. For more information about
    /// promotional offers, see Implementing promotional offers in your app.
    OfferRedeemed,
    /// A notification type that, along with its subtype, indicates that the
    /// subscription successfully renewed. If the subtype is BILLING_RECOVERY,
    /// the expired subscription that previously failed to renew has
    /// successfully renewed. If the subtype is empty, the active subscription
    /// has successfully auto-renewed for a new transaction period. Provide the
    /// customer with access to the subscription’s content or service.
    DidRenew,
    /// A notification type that, along with its subtype, indicates that a
    /// subscription expired. If the subtype is VOLUNTARY, the subscription
    /// expired after the user disabled subscription renewal. If the subtype is
    /// BILLING_RETRY, the subscription expired because the billing retry period
    /// ended without a successful billing transaction. If the subtype is
    /// PRICE_INCREASE, the subscription expired because the customer didn’t
    /// consent to a price increase that requires customer consent. If the
    /// subtype is PRODUCT_NOT_FOR_SALE, the subscription expired because the
    /// product wasn’t available for purchase at the time the subscription
    /// attempted to renew.
    ///
    /// A notification without a subtype indicates that the subscription expired
    /// for some other reason.
    Expired,
    /// A notification type that, along with its subtype, indicates that the
    /// subscription failed to renew due to a billing issue. The subscription
    /// enters the billing retry period. If the subtype is GRACE_PERIOD,
    /// continue to provide service through the grace period. If the subtype is
    /// empty, the subscription isn’t in a grace period and you can stop
    /// providing the subscription service.
    ///
    /// Inform the customer that there may be an issue with their billing
    /// information. The App Store continues to retry billing for 60 days, or
    /// until the customer resolves their billing issue or cancels their
    /// subscription, whichever comes first. For more information, see Reducing
    /// Involuntary Subscriber Churn.
    DidFailToRenew,
    /// A notification type that indicates that the billing grace period has
    /// ended without renewing the subscription, so you can turn off access to
    /// the service or content. Inform the customer that there may be an issue
    /// with their billing information. The App Store continues to retry billing
    /// for 60 days, or until the customer resolves their billing issue or
    /// cancels their subscription, whichever comes first. For more information,
    /// see Reducing Involuntary Subscriber Churn.
    GracePeriodExpired,
    /// A notification type that, along with its subtype, indicates that the
    /// system has informed the customer of an auto-renewable subscription price
    /// increase.
    ///
    /// If the price increase requires customer consent, the subtype is PENDING
    /// if the customer hasn’t responded to the price increase, or ACCEPTED if
    /// the customer has consented to the price increase.
    ///
    /// If the price increase doesn’t require customer consent, the subtype is
    /// ACCEPTED.
    ///
    /// For information about how the system calls your app before it displays
    /// the price consent sheet for subscription price increases that require
    /// customer consent, see paymentQueueShouldShowPriceConsent(_:). For
    /// information about managing subscription prices, see Managing Price
    /// Increases for Auto-Renewable Subscriptions and Managing Prices.
    PriceIncrease,
    /// A notification type that indicates that the App Store successfully
    /// refunded a transaction for a consumable in-app purchase, a
    /// non-consumable in-app purchase, an auto-renewable subscription, or a
    /// non-renewing subscription.
    ///
    /// The revocationDate contains the timestamp of the refunded transaction.
    /// The originalTransactionId and productId identify the original
    /// transaction and product. The revocationReason contains the reason.
    ///
    /// To request a list of all refunded transactions for a customer, see Get
    /// Refund History in the App Store Server API.
    Refund,
    /// A notification type that indicates the App Store declined a refund
    /// request.
    RefundDeclined,
    /// A notification type that indicates the App Store reversed a previously
    /// granted refund due to a dispute that the customer raised. If your app
    /// revoked content or services as a result of the related refund, it needs
    /// to reinstate them.
    ///
    /// This notification type can apply to any in-app purchase type:
    /// consumable, non-consumable, non-renewing subscription, and
    /// auto-renewable subscription. For auto-renewable subscriptions, the
    /// renewal date remains unchanged when the App Store reverses a refund.
    RefundReversed,
    /// A notification type that indicates that the App Store extended the
    /// subscription renewal date for a specific subscription. You request
    /// subscription-renewal-date extensions by calling Extend a Subscription
    /// Renewal Date or Extend Subscription Renewal Dates for All Active
    /// Subscribers in the App Store Server API.
    RenewalExtended,
    /// A notification type that, along with its subtype, indicates that the App
    /// Store is attempting to extend the subscription renewal date that you
    /// request by calling Extend Subscription Renewal Dates for All Active
    /// Subscribers.
    ///
    /// If the subtype is SUMMARY, the App Store completed extending the renewal
    /// date for all eligible subscribers. See the summary in the
    /// responseBodyV2DecodedPayload for details. If the subtype is FAILURE, the
    /// renewal date extension didn’t succeed for a specific subscription. See
    /// the data in the responseBodyV2DecodedPayload for details.
    RenewalExtension,
    /// A notification type that indicates that an in-app purchase the customer
    /// was entitled to through Family Sharing is no longer available through
    /// sharing. The App Store sends this notification when a purchaser disables
    /// Family Sharing for their purchase, the purchaser (or family member)
    /// leaves the family group, or the purchaser receives a refund. Your app
    /// also receives a
    /// paymentQueue(_:didRevokeEntitlementsForProductIdentifiers:) call. Family
    /// Sharing applies to non-consumable in-app purchases and auto-renewable
    /// subscriptions. For more information about Family Sharing, see Supporting
    /// Family Sharing in your app.
    Revoke,
    /// A notification type that the App Store server sends when you request it
    /// by calling the Request a Test Notification endpoint. Call that endpoint
    /// to test whether your server is receiving notifications. You receive this
    /// notification only at your request. For troubleshooting information, see
    /// the Get Test Notification Status endpoint.
    Test,
    /// A notification type that, along with its subtype UNREPORTED, indicates
    /// that Apple created an external purchase token for your app, but didn’t
    /// receive a report. For more information about reporting the token, see
    /// externalPurchaseToken.
    ///
    /// This notification applies only to apps that use the External Purchase to
    /// provide alternative payment options.
    ExternalPurchaseToken,
    /// Currently available only in the sandbox environment.
    ///
    /// A notification type that indicates the customer purchased a consumable,
    /// non-consumable, or non-renewing subscription. The App Store also sends
    /// this notification when the customer receives access to a non-consumable
    /// product through Family Sharing.
    ///
    /// For notifications about auto-renewable subscription purchases, see the
    /// SUBSCRIBED notification type.
    OneTimeCharge,
    /// A notification type that indicates that the customer initiated a refund
    /// request for a consumable in-app purchase or auto-renewable subscription,
    /// and the App Store is requesting that you provide consumption data. For
    /// more information, see Send Consumption Information.
    ConsumptionRequest,

    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum NotificationSubtype {
    /// Applies to the SUBSCRIBED notificationType. A notification with this
    /// subtype indicates that the user purchased the subscription for the first
    /// time or that the user received access to the subscription through Family
    /// Sharing for the first time.
    InitialBuy,
    /// Applies to the SUBSCRIBED notificationType. A notification with this
    /// subtype indicates that the user resubscribed or received access through
    /// Family Sharing to the same subscription or to another subscription
    /// within the same subscription group.
    Resubscribe,
    /// Applies to the DID_CHANGE_RENEWAL_PREF and OFFER_REDEEMED
    /// notificationType. A notification with this subtype indicates that the
    /// user downgraded their subscription or cross-graded to a subscription
    /// with a different duration. Downgrades take effect at the next renewal
    /// date.
    Downgrade,
    /// Applies to the DID_CHANGE_RENEWAL_PREF and OFFER_REDEEMED
    /// notificationType. A notification with this subtype indicates that the
    /// user upgraded their subscription or cross-graded to a subscription with
    /// the same duration. Upgrades take effect immediately.
    Upgrade,
    /// Applies to the DID_CHANGE_RENEWAL_STATUS notificationType. A
    /// notification with this subtype indicates that the user enabled
    /// subscription auto-renewal.
    AutoRenewEnabled,
    /// Applies to the DID_CHANGE_RENEWAL_STATUS notificationType. A
    /// notification with this subtype indicates that the user disabled
    /// subscription auto-renewal, or the App Store disabled subscription
    /// auto-renewal after the user requested a refund.
    AutoRenewDisabled,
    /// Applies to the EXPIRED notificationType. A notification with this
    /// subtype indicates that the subscription expired after the user disabled
    /// subscription auto-renewal.
    Voluntary,
    /// Applies to the EXPIRED notificationType. A notification with this
    /// subtype indicates that the subscription expired because the subscription
    /// failed to renew before the billing retry period ended.
    BillingRetry,
    /// Applies to the EXPIRED notificationType. A notification with this
    /// subtype indicates that the subscription expired because the user didn’t
    /// consent to a price increase.
    PriceIncrease,
    /// Applies to the DID_FAIL_TO_RENEW notificationType. A notification with
    /// this subtype indicates that the subscription failed to renew due to a
    /// billing issue. Continue to provide access to the subscription during the
    /// grace period.
    GracePeriod,
    /// Applies to the PRICE_INCREASE notificationType. A notification with this
    /// subtype indicates that the system informed the user of the subscription
    /// price increase, but the user hasn’t accepted it.
    Pending,
    /// Applies to the PRICE_INCREASE notificationType. A notification with this
    /// subtype indicates that the customer consented to the subscription price
    /// increase if the price increase requires customer consent, or that the
    /// system notified them of a price increase if the price increase doesn’t
    /// require customer consent.
    Accepted,
    /// Applies to the DID_RENEW notificationType. A notification with this
    /// subtype indicates that the expired subscription that previously failed
    /// to renew has successfully renewed.
    BillingRecovery,
    /// Applies to the EXPIRED notificationType. A notification with this
    /// subtype indicates that the subscription expired because the product
    /// wasn’t available for purchase at the time the subscription attempted to
    /// renew.
    ProductNotForSale,
    /// Applies to the RENEWAL_EXTENSION notificationType. A notification with
    /// this subtype indicates that the App Store server completed your request
    /// to extend the subscription renewal date for all eligible subscribers.
    /// For the summary details, see the summary object in the
    /// responseBodyV2DecodedPayload. For information on the request, see Extend
    /// Subscription Renewal Dates for All Active Subscribers.
    Summary,
    /// Applies to the RENEWAL_EXTENSION notificationType. A notification with
    /// this subtype indicates that the subscription-renewal-date extension
    /// failed for an individual subscription. For details, see the data object
    /// in the responseBodyV2DecodedPayload. For information on the request, see
    /// Extend Subscription Renewal Dates for All Active Subscribers.
    Failure,
    /// Applies to the EXTERNAL_PURCHASE_TOKEN notificationType. A notification
    /// with this subtype indicates that Apple created a token for your app but
    /// didn’t receive a report. For more information about reporting the token,
    /// see externalPurchaseToken.
    Unreported,

    #[serde(untagged)]
    Unknown(String),
}

/// The payload data that contains app metadata and the signed renewal and
/// transaction information. App Store Server Notifications 1.0+
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotificationData {
    /// The unique identifier of the app that the notification applies to. This
    /// property is available for apps that users download from the App Store.
    /// It isn’t present in the sandbox environment.
    pub(crate) app_apple_id: Option<AppleIdType>,
    /// The bundle identifier of the app.
    pub(crate) bundle_id: String,
    /// The version of the build that identifies an iteration of the bundle.
    pub(crate) bundle_version: Option<String>,
    /// The reason the customer requested the refund. This field appears only
    /// for CONSUMPTION_REQUEST notifications, which the server sends when a
    /// customer initiates a refund request for a consumable in-app purchase or
    /// auto-renewable subscription.
    pub(crate) consumption_request_reason: Option<ConsumptionRequestReason>,
    /// The server environment that the notification applies to, either sandbox
    /// or production.
    pub(crate) environment: Environment,
    /// Subscription renewal information signed by the App Store, in JSON Web
    /// Signature (JWS) format. This field appears only for notifications that
    /// apply to auto-renewable subscriptions.
    pub(crate) signed_renewal_info: Option<JWSRenewalInfo>,
    /// Transaction information signed by the App Store, in JSON Web Signature
    /// (JWS) format.
    pub(crate) signed_transaction_info: Option<JWSTransaction>,
    /// The status of an auto-renewable subscription as of the signedDate in the
    /// responseBodyV2DecodedPayload. This field appears only for notifications
    /// sent for auto-renewable subscriptions.
    pub(crate) status: Option<SubscriptionStatus>,
}

/// The payload data for a subscription-renewal-date extension notification.
/// App Store Server Notifications 1.0+
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotificationSummary {
    /// The UUID that represents a specific request to extend a subscription
    /// renewal date. This value matches the value you initially specify in the
    /// requestIdentifier when you call Extend Subscription Renewal Dates for
    /// All Active Subscribers in the App Store Server API.
    pub(crate) request_identifier: String,
    /// The server environment that the notification applies to, either sandbox
    /// or production.
    pub(crate) environment: Environment,
    /// The unique identifier of the app that the notification applies to. This
    /// property is available for apps that users download from the App Store.
    /// It isn’t present in the sandbox environment.
    pub(crate) app_apple_id: Option<AppleIdType>,
    /// The bundle identifier of the app.
    pub(crate) bundle_id: String,
    /// The product identifier of the auto-renewable subscription that the
    /// subscription-renewal-date extension applies to.
    pub(crate) product_id: String,
    /// A list of country codes that limits the App Store’s attempt to apply the
    /// subscription-renewal-date extension. If this list isn’t present, the
    /// subscription-renewal-date extension applies to all storefronts.
    #[serde(default)]
    pub(crate) storefront_country_codes: Vec<String>,
    /// The final count of subscriptions that fail to receive a
    /// subscription-renewal-date extension.
    #[serde(default)]
    pub(crate) failed_count: i64,
    /// The final count of subscriptions that successfully receive a
    /// subscription-renewal-date extension.
    #[serde(default)]
    pub(crate) succeeded_count: i64,
}

/// The payload data that contains an external purchase token. App Store Server
/// Notifications 1.0+
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExternalPurchaseToken {
    /// The unique identifier of the token. Use this value to report tokens and
    /// their associated transactions in the Send External Purchase Report
    /// endpoint.
    pub(crate) external_purchase_id: String,
    /// The UNIX time, in milliseconds, when the system created the token.
    #[serde(with = "ts_milliseconds")]
    pub(crate) token_creation_date: DateTime<Utc>,
    /// The app Apple ID for which the system generated the token.
    pub(crate) app_apple_id: Option<AppleIdType>,
    /// The bundle ID of the app for which the system generated the token.
    pub(crate) bundle_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConsumptionRequestReason {
    /// The customer didn’t intend to make the in-app purchase.
    UnintendedPurchase,
    /// The customer had issues with receiving or using the in-app purchase.
    FulfillmentIssue,
    /// The customer wasn’t satisfied with the in-app purchase.
    UnsatisfiedWithPurchase,
    /// The customer requested a refund based on a legal reason.
    Legal,
    /// The customer requested a refund for other reasons.
    Other,

    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
pub enum SubscriptionStatus {
    /// The auto-renewable subscription is active.
    Active = 1,
    /// The auto-renewable subscription is expired.
    Expired = 2,
    /// The auto-renewable subscription is in a billing retry period.
    BillingRetry = 3,
    /// The auto-renewable subscription is in a Billing Grace Period.
    BillingGracePeriod = 4,
    /// The auto-renewable subscription is revoked.
    Revoked = 5,
}
