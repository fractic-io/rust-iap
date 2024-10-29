use fractic_aws_secrets::SecretValues;
use fractic_generic_server_error::GenericServerError;

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
    pub async fn verify_and_get_details<T: TypedProductId>(
        &self,
        product_id: T,
        purchase_id: IapPurchaseId,
        include_price_info: bool,
    ) -> Result<IapDetails<T::DetailsType>, GenericServerError> {
        self.iap_repository
            .verify_and_get_details(product_id, purchase_id, include_price_info)
            .await
    }

    pub async fn parse_apple_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError> {
        self.iap_repository.parse_apple_notification(body).await
    }

    pub async fn parse_google_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError> {
        self.iap_repository.parse_google_notification(body).await
    }
}

impl IapUtil {
    pub async fn from_secrets(
        secrets: SecretValues<IapSecretsConfig>,
        application_id: String,
    ) -> Result<Self, GenericServerError> {
        Ok(Self {
            iap_repository: IapRepositoryImpl::new(
                application_id,
                secrets.get(&IapSecretsConfig::AppleApiKey)?,
                secrets.get(&IapSecretsConfig::AppleKeyId)?,
                secrets.get(&IapSecretsConfig::AppleIssuerId)?,
                secrets.get(&IapSecretsConfig::GoogleApiKey)?,
            )
            .await?,
        })
    }

    pub async fn from_values(
        application_id: String,
        apple_api_key: &str,
        apple_key_id: &str,
        apple_issuer_id: &str,
        google_api_key: &str,
    ) -> Result<Self, GenericServerError> {
        Ok(Self {
            iap_repository: IapRepositoryImpl::new(
                application_id,
                apple_api_key,
                apple_key_id,
                apple_issuer_id,
                google_api_key,
            )
            .await?,
        })
    }
}
