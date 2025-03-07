use chrono::{DateTime, Utc};

use super::iap_purchase_id::IapPurchaseId;

#[derive(Debug, Clone, PartialEq)]
pub enum MaybeKnown<T> {
    Known(T),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PriceInfo {
    /// The price in micro-units, where 1,000,000 micro-units equal one unit of
    /// the currency.
    pub price_micros: i64,
    /// 3-letter ISO 4217 currency code.
    pub currency_iso_4217: String,
}

#[derive(Debug, Clone)]
pub struct IapDetails<T: IapTypeSpecificDetails> {
    pub cannonical_id: IapPurchaseId,
    pub is_active: bool,
    pub is_sandbox: bool,
    pub is_finalized_by_client: MaybeKnown<bool>,
    pub purchase_time: DateTime<Utc>,
    pub region_iso3166_alpha_3: String,
    pub price_info: Option<PriceInfo>,

    pub type_specific_details: T,
}

pub trait IapTypeSpecificDetails: Send + Sync {}
impl IapTypeSpecificDetails for NonConsumableDetails {}
impl IapTypeSpecificDetails for ConsumableDetails {}
impl IapTypeSpecificDetails for SubscriptionDetails {}

#[derive(Debug, Clone)]
pub struct NonConsumableDetails {}

#[derive(Debug, Clone)]
pub struct ConsumableDetails {
    pub is_consumed: MaybeKnown<bool>,
    pub quantity: i64,
}

#[derive(Debug, Clone)]
pub struct SubscriptionDetails {
    pub expiration_time: DateTime<Utc>,
}

pub trait IapGenericDetails {
    fn is_active(&self) -> bool;
    fn is_sandbox(&self) -> bool;
    fn is_finalized_by_client(&self) -> MaybeKnown<bool>;
    fn purchase_time(&self) -> DateTime<Utc>;
    fn region_iso3166_alpha_3(&self) -> &str;
    fn price_info(&self) -> Option<&PriceInfo>;
}

impl<T: IapTypeSpecificDetails> IapGenericDetails for IapDetails<T> {
    fn is_active(&self) -> bool {
        self.is_active
    }

    fn is_sandbox(&self) -> bool {
        self.is_sandbox
    }

    fn is_finalized_by_client(&self) -> MaybeKnown<bool> {
        self.is_finalized_by_client.clone()
    }

    fn purchase_time(&self) -> DateTime<Utc> {
        self.purchase_time
    }

    fn region_iso3166_alpha_3(&self) -> &str {
        &self.region_iso3166_alpha_3
    }

    fn price_info(&self) -> Option<&PriceInfo> {
        self.price_info.as_ref()
    }
}
