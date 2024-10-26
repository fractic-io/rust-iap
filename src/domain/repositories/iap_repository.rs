use fractic_generic_server_error::GenericServerError;

use crate::domain::entities::{
    iap_details::IapDetails, iap_id::IapId, iap_type::IapType,
    iap_update_notification::IapUpdateNotification,
};

pub trait IapRepository {
    fn verify_and_get_details(
        &self,
        id: IapId,
        product_type: IapType,
    ) -> Result<IapDetails, GenericServerError>;

    fn parse_apple_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError>;

    fn parse_google_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError>;
}
