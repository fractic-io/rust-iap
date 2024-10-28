#[derive(Debug)]
pub enum IapPurchaseId {
    AppStoreTransactionId(String),
    GooglePlayPurchaseToken(String),
}
