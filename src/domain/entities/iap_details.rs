use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
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
