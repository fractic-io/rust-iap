use fractic_env_config::SecretValues;
use fractic_server_error::ServerError;

use crate::{
    data::{
        datasources::{
            app_store_server_api_datasource::AppStoreServerApiDatasourceImpl,
            app_store_server_notification_datasource::AppStoreServerNotificationDatasourceImpl,
            google_cloud_rtdn_notification_datasource::GoogleCloudRtdnNotificationDatasourceImpl,
            google_play_developer_api_datasource::GooglePlayDeveloperApiDatasourceImpl,
        },
        repositories::iap_repository_impl::IapRepositoryImpl,
    },
    domain::{
        entities::{
            iap_details::IapDetails, iap_purchase_id::IapPurchaseId,
            iap_update_notification::IapUpdateNotification,
        },
        repositories::iap_repository::{IapRepository, TypedProductId},
    },
    secrets::IapSecretsConfig,
};

pub struct IapUtil {
    iap_repository: IapRepositoryImpl<
        AppStoreServerApiDatasourceImpl,
        AppStoreServerNotificationDatasourceImpl,
        GooglePlayDeveloperApiDatasourceImpl,
        GoogleCloudRtdnNotificationDatasourceImpl,
    >,
}

impl IapUtil {
    /// Verify the authenticity of a purchase, and return the purchase details retrieved
    /// from the respective platform's API, abstracted into a platform-generic struct.
    ///
    /// If 'include_price_info' is true, the price and currency information will
    /// also be populated. For Google Play purchases, this requires an
    /// additional callout.
    ///
    /// This callout will fail if the purchase does not exist, or if it is not
    /// in an active state (ex. voided or subscription cancelled).
    pub async fn verify_and_get_details<T: TypedProductId>(
        &self,
        product_id: T,
        purchase_id: IapPurchaseId,
        include_price_info: bool,
    ) -> Result<IapDetails<T::DetailsType>, ServerError> {
        self.iap_repository
            .verify_and_get_details(product_id, purchase_id, include_price_info)
            .await
    }

    /// Verify the notification authenticity (signed by Apple), and parse body
    /// into a generic update notification.
    ///
    /// NOTE: To verify Apple's signature, this function calls out to Apple's
    /// OAuth endpoint.
    pub async fn parse_apple_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, ServerError> {
        self.iap_repository.parse_apple_notification(body).await
    }

    /// Verify the notification authenticity (signed by Google), and parse body
    /// into a generic update notification.
    ///
    /// NOTE: To verify Google's signature, this function calls out to Google's
    /// OAuth endpoint.
    pub async fn parse_google_notification(
        &self,
        authorization_header: &str,
        body: &str,
    ) -> Result<IapUpdateNotification, ServerError> {
        self.iap_repository
            .parse_google_notification(authorization_header, body)
            .await
    }

    /// Request a server-to-server notification of type 'TEST' from Apple.
    ///
    /// Currently, the only way to request test notifications from Apple is
    /// through the API. For Google Play, one can simply request test
    /// notifications in the console.
    pub async fn request_apple_test_notification(&self, sandbox: bool) -> Result<(), ServerError> {
        self.iap_repository
            .request_apple_test_notification(sandbox)
            .await
    }
}

impl IapUtil {
    pub async fn from_secrets(
        secrets: SecretValues<IapSecretsConfig>,
        application_id: impl Into<String>,
        aud_claim: impl Into<String>,
    ) -> Result<Self, ServerError> {
        Ok(Self {
            iap_repository: IapRepositoryImpl::new(
                application_id,
                aud_claim,
                secrets.get(&IapSecretsConfig::AppleApiKey)?,
                secrets.get(&IapSecretsConfig::AppleKeyId)?,
                secrets.get(&IapSecretsConfig::AppleIssuerId)?,
                secrets.get(&IapSecretsConfig::GoogleApiKey)?,
            )
            .await?,
        })
    }

    pub async fn from_values(
        application_id: impl Into<String>,
        expected_aud: impl Into<String>,
        apple_api_key: &str,
        apple_key_id: &str,
        apple_issuer_id: &str,
        google_api_key: &str,
    ) -> Result<Self, ServerError> {
        Ok(Self {
            iap_repository: IapRepositoryImpl::new(
                application_id,
                expected_aud,
                apple_api_key,
                apple_key_id,
                apple_issuer_id,
                google_api_key,
            )
            .await?,
        })
    }
}
