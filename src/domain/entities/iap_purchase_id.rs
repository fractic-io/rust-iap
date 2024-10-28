#[derive(Debug, Clone)]
pub enum IapPurchaseId {
    AppStoreTransactionId(String),
    GooglePlayPurchaseToken(String),
}
