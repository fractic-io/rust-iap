use chrono::{DateTime, Utc};

use super::{
    iap_details::{ConsumableDetails, IapDetails, NonConsumableDetails, SubscriptionDetails},
    iap_product_id::{IapConsumableId, IapNonConsumableId, IapSubscriptionId},
    iap_purchase_id::IapPurchaseId,
};

#[derive(Debug, Clone)]
pub struct IapUpdateNotification {
    pub notification_id: String,
    pub time: DateTime<Utc>,
    pub details: NotificationDetails,
}

#[derive(Debug, Clone)]
pub enum NotificationDetails {
    Test,
    ConsumableVoided {
        application_id: String,
        product_id: IapConsumableId,
        purchase_id: IapPurchaseId,
        details: IapDetails<ConsumableDetails>,
        is_refunded: bool,
        reason: Option<String>,
    },
    NonConsumableVoided {
        application_id: String,
        product_id: IapNonConsumableId,
        purchase_id: IapPurchaseId,
        details: IapDetails<NonConsumableDetails>,
        is_refunded: bool,
        reason: Option<String>,
    },
    UnknownOneTimePurchaseVoided {
        application_id: String,
        purchase_id: IapPurchaseId,
        is_refunded: bool,
        reason: Option<String>,
    },
    SubscriptionStarted {
        application_id: String,
        product_id: IapSubscriptionId,
        purchase_id: IapPurchaseId,
        details: IapDetails<SubscriptionDetails>,
    },
    SubscriptionEnded {
        application_id: String,
        product_id: IapSubscriptionId,
        purchase_id: IapPurchaseId,
        details: IapDetails<SubscriptionDetails>,
        reason: SubscriptionEndReason,
    },
    /// Any events that change the expiry of the product, most commonly renewal,
    /// but also includes things like grace periods.
    SubscriptionExpiryChanged {
        application_id: String,
        product_id: IapSubscriptionId,
        purchase_id: IapPurchaseId,
        details: IapDetails<SubscriptionDetails>,
    },
    Other,
}

#[derive(Debug, Clone)]
pub enum SubscriptionEndReason {
    Paused,
    Cancelled { details: Option<String> },
    FailedToRenew,
    Voided { is_refunded: bool },
    DeclinedPriceIncrease,
    Unknown,
}
