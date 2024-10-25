#![allow(dead_code)]

use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Debug, Deserialize)]
pub(crate) enum Environment {
    /// Indicates that the data applies to testing in the sandbox environment.
    Sandbox,
    /// Indicates that the data applies to the production environment.
    Production,

    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum OfferDiscountType {
    /// A payment mode of a product discount that indicates a free trial.
    FreeTrial,
    /// A payment mode of a product discount that customers pay over a single or
    /// multiple billing periods.
    PayAsYouGo,
    /// A payment mode of a product discount that customers pay up front.
    PayUpFront,

    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
pub(crate) enum OfferType {
    /// An introductory offer.
    Introductory = 1,
    /// A promotional offer.
    Promotional = 2,
    /// An offer with a subscription offer code.
    OfferCode = 3,
    /// A win-back offer.
    WinBack = 4,
}
