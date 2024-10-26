use super::{iap_details::IapDetails, iap_id::IapId};

#[derive(Debug)]
pub struct IapUpdateNotification {
    pub id: IapId,
    pub details: IapDetails,
}
