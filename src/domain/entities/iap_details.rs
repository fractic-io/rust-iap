use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct IapDetails<T: IapTypeSpecificDetails> {
    pub is_active: bool,
    pub purchase_time: DateTime<Utc>,

    pub type_specific_details: T,
}

pub trait IapTypeSpecificDetails: Send + Sync {}
impl IapTypeSpecificDetails for SubscriptionDetails {}
impl IapTypeSpecificDetails for ConsumableDetails {}
impl IapTypeSpecificDetails for NonConsumableDetails {}

#[derive(Debug)]
pub struct SubscriptionDetails {
    pub expiration_time: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ConsumableDetails {
    pub is_consumed: bool,
}

#[derive(Debug)]
pub struct NonConsumableDetails {}
