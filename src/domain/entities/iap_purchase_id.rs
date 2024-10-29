#[derive(Debug, Clone)]
pub enum IapPurchaseId {
    /// The transaction ID from the Apple App Store.
    ///
    /// In the case of subscriptions, this should always be the 'original'
    /// transaction ID, not the transaction ID of the latest renewal.
    AppStoreTransactionId(String),

    /// Purchase token received on the device when purchasing an in-app-purchase
    /// with the Google Play Store.
    ///
    /// In the case of subscriptions, this ID does not change accross renewals.
    GooglePlayPurchaseToken(String),
}
