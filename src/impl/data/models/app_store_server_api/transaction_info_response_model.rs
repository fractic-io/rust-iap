#![allow(dead_code)]

use serde::Deserialize;

type JWSTransaction = String;

/// Data structure returned by the App Store Server API when querying for
/// transaction info.
///
/// https://developer.apple.com/documentation/appstoreserverapi/transactioninforesponse
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransactionInfoResponseModel {
    /// A customer’s in-app purchase transaction, signed by Apple, in JSON Web
    /// Signature (JWS) format.
    pub(crate) signed_transaction_info: JWSTransaction,
}
