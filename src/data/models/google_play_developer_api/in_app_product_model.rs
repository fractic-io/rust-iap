#![allow(dead_code)]

use std::collections::HashMap;

use serde::Deserialize;

/// Data structure returned by the Google Play Developer API when querying for
/// an in-app product.
///
/// https://developers.google.com/android-publisher/api-ref/rest/v3/inappproducts#InAppProduct
///
/// Whether fields are nullable is not documented explicitly in the API
/// reference, so reasonable assumptions are made.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InAppProductModel {
    /// Package name of the parent app.
    pub(crate) package_name: String,
    /// Stock-keeping-unit (SKU) of the product, unique within an app.
    pub(crate) sku: String,
    /// The status of the product, e.g. whether it's active.
    pub(crate) status: Status,
    /// The type of the product, e.g. a recurring subscription.
    pub(crate) purchase_type: PurchaseType,
    /// Default price. Cannot be zero, as in-app products are never free. Always
    /// in the developer's Checkout merchant currency.
    pub(crate) default_price: Price,
    /// Prices per buyer region. None of these can be zero, as in-app products
    /// are never free. Map key is region code, as defined by ISO 3166-2.
    #[serde(default)]
    pub(crate) prices: HashMap<String, Price>,
    /// inappproducts.list of localized title and description data. Map key is
    /// the language of the localized data, as defined by BCP-47, e.g. "en-US".
    #[serde(default)]
    pub(crate) listings: HashMap<String, InAppProductListing>,
    /// Default language of the localized data, as defined by BCP-47. e.g.
    /// "en-US".
    pub(crate) default_language: String,
    /// Subscription period, specified in ISO 8601 format. Acceptable values are
    /// P1W (one week), P1M (one month), P3M (three months), P6M (six months),
    /// and P1Y (one year).
    pub(crate) subscription_period: Option<String>,
    /// Trial period, specified in ISO 8601 format. Acceptable values are
    /// anything between P7D (seven days) and P999D (999 days).
    pub(crate) trial_period: Option<String>,
    /// Grace period of the subscription, specified in ISO 8601 format. Allows
    /// developers to give their subscribers a grace period when the payment for
    /// the new recurrence period is declined. Acceptable values are P0D (zero
    /// days), P3D (three days), P7D (seven days), P14D (14 days), and P30D (30
    /// days).
    pub(crate) grace_period: Option<String>,
    //
    // Can implement if needed in future:
    // // Union field TaxAndComplianceType can be only one of the following:
    // // --
    // /// Details about taxes and legal compliance. Only applicable to
    // /// subscription products.
    // pub(crate) subscription_taxes_and_compliance_settings:
    //     Option<SubscriptionTaxesAndComplianceSettings>,
    // /// Details about taxes and legal compliance. Only applicable to managed
    // /// products.
    // pub(crate) managed_product_taxes_and_compliance_settings:
    //     Option<ManagedProductTaxesAndComplianceSettings>,
    // // --
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Status {
    /// Unspecified status.
    StatusUnspecified,
    /// The product is published and active in the store.
    Active,
    /// The product is not published and therefore inactive in the store.
    Inactive,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PurchaseType {
    /// Unspecified purchase type.
    PurchaseTypeUnspecified,
    /// The default product type - one time purchase.
    ManagedUser,
    /// In-app product with a recurring period.
    Subscription,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    /// Price in 1/million of the currency base unit, represented as a string.
    pub(crate) price_micros: String,
    /// 3 letter Currency code, as defined by ISO 4217. See
    /// java/com/google/common/money/CurrencyCode.java
    pub(crate) currency: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InAppProductListing {
    /// Title for the store listing.
    pub(crate) title: String,
    /// Description for the store listing.
    pub(crate) description: String,
    /// Localized entitlement benefits for a subscription.
    #[serde(default)]
    pub(crate) benefits: Vec<String>,
}
