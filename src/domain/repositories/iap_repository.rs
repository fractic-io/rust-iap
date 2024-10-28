use async_trait::async_trait;
use fractic_generic_server_error::GenericServerError;

use crate::{
    data::models::{
        app_store_server_api::jws_transaction_decoded_payload_model::JwsTransactionDecodedPayloadModel,
        google_play_developer_api::{
            product_purchase_model::ProductPurchaseModel,
            subscription_purchase_v2_model::SubscriptionPurchaseV2Model,
        },
    },
    domain::entities::{
        iap_details::{IapDetails, IapTypeSpecificDetails},
        iap_product_id::private::IapProductId,
        iap_purchase_id::IapPurchaseId,
        iap_update_notification::IapUpdateNotification,
    },
};

pub trait TypedProductId: IapProductId {
    type DetailsType: IapTypeSpecificDetails;

    fn extract_details_from_apple_transaction(
        m: &JwsTransactionDecodedPayloadModel,
    ) -> Self::DetailsType;

    fn extract_details_from_google_product_purchase(m: &ProductPurchaseModel) -> Self::DetailsType;

    fn extract_details_from_google_subscription_purchase(
        m: &SubscriptionPurchaseV2Model,
    ) -> Self::DetailsType;
}

#[async_trait]
pub trait IapRepository: Send + Sync {
    async fn verify_and_get_details<T: TypedProductId>(
        &self,
        product_id: T,
        purchase_id: IapPurchaseId,
    ) -> Result<IapDetails<T::DetailsType>, GenericServerError>;

    async fn parse_apple_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError>;

    async fn parse_google_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError>;
}
