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
            iap_details::IapDetails, iap_id::IapId, iap_type::IapType,
            iap_update_notification::IapUpdateNotification,
        },
        repositories::iap_repository::IapRepository,
    },
    secrets::IapSecretsConfig,
};

pub struct IapUtil<R: IapRepository> {
    iap_repository: R,
}

impl<R: IapRepository> IapUtil<R> {
    pub fn verify_and_get_details(
        &self,
        id: IapId,
        product_type: IapType,
    ) -> Result<IapDetails, GenericServerError> {
        self.iap_repository.verify_and_get_details(id, product_type)
    }

    pub fn parse_apple_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError> {
        self.iap_repository.parse_apple_notification(body)
    }

    pub fn parse_google_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError> {
        self.iap_repository.parse_google_notification(body)
    }
}

impl
    IapUtil<
        IapRepositoryImpl<
            AppStoreServerApiDatasourceImpl,
            AppStoreServerNotificationDatasourceImpl,
            GooglePlayDeveloperApiDatasourceImpl,
            GoogleCloudRtdnNotificationDatasourceImpl,
        >,
    >
{
    pub async fn new(
        secrets: SecretValues<IapSecretsConfig>,
        bundle_id: &str,
    ) -> Result<Self, GenericServerError> {
        Ok(Self {
            iap_repository: IapRepositoryImpl::new(
                secrets.get(&IapSecretsConfig::AppleApiKey)?,
                secrets.get(&IapSecretsConfig::AppleKeyId)?,
                secrets.get(&IapSecretsConfig::AppleIssuerId)?,
                bundle_id,
                secrets.get(&IapSecretsConfig::GoogleApiKey)?,
            )
            .await?,
        })
    }
}
