#[derive(Debug)]
pub enum IapId {
    AppStoreTransactionId(String),
    GooglePlayPurchaseToken(String),
}
