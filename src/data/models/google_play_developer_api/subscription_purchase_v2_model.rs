#![allow(dead_code)]

use serde::Deserialize;

/// Data structure returned by the Google Play Developer API when querying for a
/// subscription purchase.
///
/// https://developers.google.com/android-publisher/api-ref/rest/v3/purchases.subscriptionsv2#SubscriptionPurchaseV2
///
/// Whether fields are nullable is not documented explicitly in the API
/// reference, so reasonable assumptions are made.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionPurchaseV2Model {
    /// This kind represents a SubscriptionPurchaseV2 object in the
    /// androidpublisher service.
    pub(crate) kind: Option<String>,
    /// ISO 3166-1 alpha-2 billing country/region code of the user at the time
    /// the subscription was granted.
    pub(crate) region_code: Option<String>,
    /// Item-level info for a subscription purchase. The items in the same
    /// purchase should be either all with AutoRenewingPlan or all with
    /// PrepaidPlan.
    #[serde(default)]
    pub(crate) line_items: Vec<SubscriptionPurchaseLineItem>,
    /// Time at which the subscription was granted. Not set for pending
    /// subscriptions (subscription was created but awaiting payment during
    /// signup).
    ///
    /// A timestamp in RFC3339 UTC "Zulu" format, with nanosecond resolution and
    /// up to nine fractional digits. Examples: "2014-10-02T15:01:23Z" and
    /// "2014-10-02T15:01:23.045123456Z".
    pub(crate) start_time: Option<String>,
    /// The current state of the subscription.
    pub(crate) subscription_state: SubscriptionState,
    /// The order id of the latest order associated with the purchase of the
    /// subscription. For autoRenewing subscription, this is the order id of
    /// signup order if it is not renewed yet, or the last recurring order id
    /// (success, pending, or declined order). For prepaid subscription, this is
    /// the order id associated with the queried purchase token.
    pub(crate) latest_order_id: String,
    /// The purchase token of the old subscription if this subscription is one
    /// of the following:
    /// * Re-signup of a canceled but non-lapsed subscription.
    /// * Upgrade/downgrade from a previous subscription.
    /// * Convert from prepaid to auto renewing subscription.
    /// * Convert from an auto renewing subscription to prepaid.
    /// * Topup a prepaid subscription.
    pub(crate) linked_purchase_token: Option<String>,
    /// Additional context around paused subscriptions. Only present if the
    /// subscription currently has subscriptionState SUBSCRIPTION_STATE_PAUSED.
    pub(crate) paused_state_context: Option<PausedStateContext>,
    /// Additional context around canceled subscriptions. Only present if the
    /// subscription currently has subscriptionState SUBSCRIPTION_STATE_CANCELED
    /// or SUBSCRIPTION_STATE_EXPIRED.
    pub(crate) canceled_state_context: Option<CanceledStateContext>,
    /// Only present if this subscription purchase is a test purchase.
    pub(crate) test_purchase: Option<TestPurchase>,
    /// The acknowledgement state of the subscription.
    pub(crate) acknowledgement_state: AcknowledgementState,
    /// User account identifier in the third-party service.
    pub(crate) external_account_identifiers: Option<ExternalAccountIdentifiers>,
    /// User profile associated with purchases made with 'Subscribe with
    /// Google'.
    pub(crate) subscribe_with_google_info: Option<SubscribeWithGoogleInfo>,
}

/// The potential states a subscription can be in, for example whether it is
/// active or canceled. The items within a subscription purchase can either be
/// all auto renewing plans or prepaid plans.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum SubscriptionState {
    /// Unspecified subscription state.
    SubscriptionStateUnspecified,
    /// Subscription was created but awaiting payment during signup. In this
    /// state, all items are awaiting payment.
    SubscriptionStatePending,
    /// Subscription is active. - (1) If the subscription is an auto renewing
    /// plan, at least one item is autoRenewEnabled and not expired. - (2) If
    /// the subscription is a prepaid plan, at least one item is not expired.
    SubscriptionStateActive,
    /// Subscription is paused. The state is only available when the
    /// subscription is an auto renewing plan. In this state, all items are in
    /// paused state.
    SubscriptionStatePaused,
    /// Subscription is in grace period. The state is only available when the
    /// subscription is an auto renewing plan. In this state, all items are in
    /// grace period.
    SubscriptionStateInGracePeriod,
    /// Subscription is on hold (suspended). The state is only available when
    /// the subscription is an auto renewing plan. In this state, all items are
    /// on hold.
    SubscriptionStateOnHold,
    /// Subscription is canceled but not expired yet. The state is only
    /// available when the subscription is an auto renewing plan. All items have
    /// autoRenewEnabled set to false.
    SubscriptionStateCanceled,
    /// Subscription is expired. All items have expiryTime in the past.
    SubscriptionStateExpired,
    /// Pending transaction for subscription is canceled. If this pending
    /// purchase was for an existing subscription, use linkedPurchaseToken to
    /// get the current state of that subscription.
    SubscriptionStatePendingPurchaseCanceled,
    #[serde(untagged)]
    Unknown(String),
}

/// Information specific to a subscription in paused state.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PausedStateContext {
    /// Time at which the subscription will be automatically resumed.
    ///
    /// A timestamp in RFC3339 UTC "Zulu" format, with nanosecond resolution and
    /// up to nine fractional digits. Examples: "2014-10-02T15:01:23Z" and
    /// "2014-10-02T15:01:23.045123456Z".
    pub(crate) auto_resume_time: String,
}

/// Information specific to a subscription in the SUBSCRIPTION_STATE_CANCELED or
/// SUBSCRIPTION_STATE_EXPIRED state.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanceledStateContext {
    // Union field cancellation_reason can be only one of the following:
    // ---
    /// Subscription was canceled by user.
    pub(crate) user_initiated_cancellation: Option<UserInitiatedCancellation>,
    /// Subscription was canceled by the system, for example because of a
    /// billing problem.
    pub(crate) system_initiated_cancellation: Option<SystemInitiatedCancellation>,
    /// Subscription was canceled by the developer.
    pub(crate) developer_initiated_cancellation: Option<DeveloperInitiatedCancellation>,
    /// Subscription was replaced by a new subscription.
    pub(crate) replacement_cancellation: Option<ReplacementCancellation>,
    // ---
}

/// Information specific to cancellations initiated by users.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserInitiatedCancellation {
    /// Information provided by the user when they complete the subscription
    /// cancellation flow (cancellation reason survey).
    pub(crate) cancel_survey_result: Option<CancelSurveyResult>,
    /// The time at which the subscription was canceled by the user. The user
    /// might still have access to the subscription after this time. Use
    /// lineItems.expiry_time to determine if a user still has access.
    ///
    /// A timestamp in RFC3339 UTC "Zulu" format, with nanosecond resolution and
    /// up to nine fractional digits. Examples: "2014-10-02T15:01:23Z" and
    /// "2014-10-02T15:01:23.045123456Z".
    pub(crate) cancel_time: Option<String>,
}

/// Result of the cancel survey when the subscription was canceled by the user.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CancelSurveyResult {
    /// The reason the user selected in the cancel survey.
    pub(crate) reason: CancelSurveyReason,
    /// Only set for CANCEL_SURVEY_REASON_OTHERS. This is the user's freeform
    /// response to the survey.
    pub(crate) reason_user_input: Option<String>,
}

/// The reason the user selected in the cancel survey.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum CancelSurveyReason {
    /// Unspecified cancel survey reason.
    CancelSurveyReasonUnspecified,
    /// Not enough usage of the subscription.
    CancelSurveyReasonNotEnoughUsage,
    /// Technical issues while using the app.
    CancelSurveyReasonTechnicalIssues,
    /// Cost related issues.
    CancelSurveyReasonCostRelated,
    /// The user found a better app.
    CancelSurveyReasonFoundBetterApp,
    /// Other reasons.
    CancelSurveyReasonOthers,
    #[serde(untagged)]
    Unknown(String),
}

/// Information specific to cancellations initiated by Google system.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SystemInitiatedCancellation {}

/// Information specific to cancellations initiated by developers.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeveloperInitiatedCancellation {}

/// Information specific to cancellations caused by subscription replacement.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReplacementCancellation {}

/// Whether this subscription purchase is a test purchase.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TestPurchase {}

/// The possible acknowledgement states for a subscription.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AcknowledgementState {
    /// Unspecified acknowledgement state.
    AcknowledgementStateUnspecified,
    /// The subscription is not acknowledged yet.
    AcknowledgementStatePending,
    /// The subscription is acknowledged.
    AcknowledgementStateAcknowledged,
    #[serde(untagged)]
    Unknown(String),
}

/// User account identifier in the third-party service.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExternalAccountIdentifiers {
    /// User account identifier in the third-party service. Only present if
    /// account linking happened as part of the subscription purchase flow.
    pub(crate) external_account_id: Option<String>,
    /// An obfuscated version of the id that is uniquely associated with the
    /// user's account in your app. Present for the following purchases:
    /// * If account linking happened as part of the subscription purchase flow.
    /// * It was specified using
    /// https://developer.android.com/reference/com/android/billingclient/api/BillingFlowParams.Builder#setobfuscatedaccountid
    /// when the purchase was made.
    pub(crate) obfuscated_external_account_id: Option<String>,
    /// An obfuscated version of the id that is uniquely associated with the
    /// user's profile in your app. Only present if specified using
    /// https://developer.android.com/reference/com/android/billingclient/api/BillingFlowParams.Builder#setobfuscatedprofileid
    /// when the purchase was made.
    pub(crate) obfuscated_external_profile_id: Option<String>,
}

/// Information associated with purchases made with 'Subscribe with Google'.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscribeWithGoogleInfo {
    /// The Google profile id of the user when the subscription was purchased.
    pub(crate) profile_id: Option<String>,
    /// The profile name of the user when the subscription was purchased.
    pub(crate) profile_name: Option<String>,
    /// The email address of the user when the subscription was purchased.
    pub(crate) email_address: Option<String>,
    /// The given name of the user when the subscription was purchased.
    pub(crate) given_name: Option<String>,
    /// The family name of the user when the subscription was purchased.
    pub(crate) family_name: Option<String>,
}

/// Item-level info for a subscription purchase.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionPurchaseLineItem {
    /// The purchased product ID (for example, 'monthly001').
    pub(crate) product_id: String,
    /// Time at which the subscription expired or will expire unless the access
    /// is extended (ex. renews).
    ///
    /// A timestamp in RFC3339 UTC "Zulu" format, with nanosecond resolution and
    /// up to nine fractional digits. Examples: "2014-10-02T15:01:23Z" and
    /// "2014-10-02T15:01:23.045123456Z".
    pub(crate) expiry_time: String,
    /// The offer details for this item.
    pub(crate) offer_details: Option<OfferDetails>,
    /// Information for deferred item replacement.
    pub(crate) deferred_item_replacement: Option<DeferredItemReplacement>,

    // Union field plan_type can be only one of the following:
    // --
    /// The item is auto renewing.
    pub(crate) auto_renewing_plan: Option<AutoRenewingPlan>,
    /// The item is prepaid.
    pub(crate) prepaid_plan: Option<PrepaidPlan>,
    // --
}

/// Information related to an auto renewing plan.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AutoRenewingPlan {
    /// If the subscription is currently set to auto-renew, e.g. the user has
    /// not canceled the subscription
    pub(crate) auto_renew_enabled: bool,
    /// The information of the last price change for the item since subscription
    /// signup.
    pub(crate) price_change_details: Option<SubscriptionItemPriceChangeDetails>,
    /// The installment plan commitment and state related info for the auto
    /// renewing plan.
    pub(crate) installment_details: Option<InstallmentPlan>,
}

/// Price change related information of a subscription item.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionItemPriceChangeDetails {
    /// New recurring price for the subscription item.
    pub(crate) new_price: Money,
    /// Price change mode specifies how the subscription item price is changing.
    pub(crate) price_change_mode: PriceChangeMode,
    /// State the price change is currently in.
    pub(crate) price_change_state: PriceChangeState,
    /// The renewal time at which the price change will become effective for the
    /// user. This is subject to change(to a future time) due to cases where the
    /// renewal time shifts like pause. This field is only populated if the
    /// price change has not taken effect.
    ///
    /// A timestamp in RFC3339 UTC "Zulu" format, with nanosecond resolution and
    /// up to nine fractional digits. Examples: "2014-10-02T15:01:23Z" and
    /// "2014-10-02T15:01:23.045123456Z".
    pub(crate) expected_new_price_charge_time: Option<String>,
}

/// The mode of the price change.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum PriceChangeMode {
    /// Price change mode unspecified. This value should never be set.
    PriceChangeModeUnspecified,
    /// If the subscription price is decreasing.
    PriceDecrease,
    /// If the subscription price is increasing and the user needs to accept it.
    PriceIncrease,
    /// If the subscription price is increasing with opt out mode.
    OptOutPriceIncrease,
    #[serde(untagged)]
    Unknown(String),
}

/// The state of the price change.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum PriceChangeState {
    /// Price change state unspecified. This value should not be used.
    PriceChangeStateUnspecified,
    /// Waiting for the user to agree for the price change.
    Outstanding,
    /// The price change is confirmed to happen for the user.
    Confirmed,
    /// The price change is applied, i.e. the user has started being charged the
    /// new price.
    Applied,
    #[serde(untagged)]
    Unknown(String),
}

/// Information to a installment plan.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstallmentPlan {
    /// Total number of payments the user is initially committed for.
    pub(crate) initial_committed_payments_count: i32,
    /// Total number of payments the user will be committed for after each
    /// commitment period. Empty means the installment plan will fall back to a
    /// normal auto-renew subscription after initial commitment.
    pub(crate) subsequent_committed_payments_count: i32,
    /// Total number of committed payments remaining to be paid for in this
    /// renewal cycle.
    pub(crate) remaining_committed_payments_count: i32,
    /// If present, this installment plan is pending to be canceled. The
    /// cancellation will happen only after the user finished all committed
    /// payments.
    pub(crate) pending_cancellation: Option<PendingCancellation>,
}

/// This is an indicator of whether there is a pending cancellation on the
/// virtual installment plan. The cancellation will happen only after the user
/// finished all committed payments.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingCancellation {}

/// Information related to a prepaid plan.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PrepaidPlan {
    /// If present, this is the time after which top up purchases are allowed
    /// for the prepaid plan. Will not be present for expired prepaid plans.
    ///
    /// A timestamp in RFC3339 UTC "Zulu" format, with nanosecond resolution and
    /// up to nine fractional digits. Examples: "2014-10-02T15:01:23Z" and
    /// "2014-10-02T15:01:23.045123456Z".
    pub(crate) allow_extend_after_time: Option<String>,
}

/// Offer details information related to a purchase line item.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OfferDetails {
    /// The latest offer tags associated with the offer. It includes tags
    /// inherited from the base plan.
    #[serde(default)]
    pub(crate) offer_tags: Vec<String>,
    /// The base plan ID. Present for all base plan and offers.
    pub(crate) base_plan_id: Option<String>,
    /// The offer ID. Only present for discounted offers.
    pub(crate) offer_id: Option<String>,
}

/// Information related to deferred item replacement.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeferredItemReplacement {
    /// The productId going to replace the existing productId.
    pub(crate) product_id: Option<String>,
}

/// Represents an amount of money with its currency type.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Money {
    /// The three-letter currency code defined in ISO 4217.
    pub(crate) currency_code: String,
    /// The whole units of the amount. For example if currencyCode is "USD",
    /// then 1 unit is one US dollar.
    pub(crate) units: i64,
    /// Number of nano (10^-9) units of the amount. The value must be between
    /// -999,999,999 and +999,999,999 inclusive. If units is positive, nanos
    /// must be positive or zero. If units is zero, nanos can be positive, zero,
    /// or negative. If units is negative, nanos must be negative or zero. For
    /// example $-1.75 is represented as units=-1 and nanos=-750,000,000.
    pub(crate) nanos: i32,
}
