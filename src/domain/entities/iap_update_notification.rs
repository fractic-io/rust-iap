use super::{
    iap_details::{ConsumableDetails, IapDetails},
    iap_purchase_id::IapPurchaseId,
};

#[derive(Debug)]
pub enum IapUpdateNotification {
    TestConsumable {
        purchase_id: Option<IapPurchaseId>,
        details: Option<IapDetails<ConsumableDetails>>,
    },
}
