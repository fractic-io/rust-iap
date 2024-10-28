use chrono::{DateTime, Utc};

use super::{
    iap_details::{ConsumableDetails, IapDetails, NonConsumableDetails, SubscriptionDetails},
    iap_product_id::{IapConsumableId, IapNonConsumableId, IapSubscriptionId},
};

#[derive(Debug)]
pub struct IapUpdateNotification {
    pub notification_id: String,
    pub application_id: String,
    pub time: DateTime<Utc>,
    pub details: NotificationDetails,
}

#[derive(Debug)]
pub enum NotificationDetails {
    Test,
    ConsumableVoided {
        purchase_id: IapConsumableId,
        details: IapDetails<ConsumableDetails>,
        is_refunded: bool,
    },
    NonConsumableVoided {
        purchase_id: IapNonConsumableId,
        details: IapDetails<NonConsumableDetails>,
        is_refunded: bool,
    },
    SubscriptionEnded {
        purchase_id: IapSubscriptionId,
        details: IapDetails<SubscriptionDetails>,
        reason: SubscriptionEndReason,
    },
    SubscriptionRenewed {
        purchase_id: IapSubscriptionId,
        details: IapDetails<SubscriptionDetails>,
    },
    Other,
}

#[derive(Debug)]
pub enum SubscriptionEndReason {
    Paused,
    Cancelled { reason: Option<String> },
    FailedToRenew,
    Voided { is_refunded: bool },
    Unknown,
}
