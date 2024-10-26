use fractic_generic_server_error::GenericServerError;

use crate::{
    data::datasources::{
        app_store_server_api_datasource::{
            AppStoreServerApiDatasource, AppStoreServerApiDatasourceImpl,
        },
        app_store_server_notification_datasource::{
            AppStoreServerNotificationDatasource, AppStoreServerNotificationDatasourceImpl,
        },
        google_cloud_rtdn_notification_datasource::{
            GoogleCloudRtdnNotificationDatasource, GoogleCloudRtdnNotificationDatasourceImpl,
        },
        google_play_developer_api_datasource::{
            GooglePlayDeveloperApiDatasource, GooglePlayDeveloperApiDatasourceImpl,
        },
    },
    domain::{
        entities::{
            iap_details::IapDetails, iap_id::IapId, iap_type::IapType,
            iap_update_notification::IapUpdateNotification,
        },
        repositories::iap_repository::IapRepository,
    },
};

pub(crate) struct IapRepositoryImpl<
    A: AppStoreServerApiDatasource,
    B: AppStoreServerNotificationDatasource,
    C: GooglePlayDeveloperApiDatasource,
    D: GoogleCloudRtdnNotificationDatasource,
> {
    app_store_server_api_datasource: A,
    app_store_server_notification_datasource: B,
    google_play_developer_api_datasource: C,
    google_cloud_rtdn_notification_datasource: D,
}

impl<
        A: AppStoreServerApiDatasource,
        B: AppStoreServerNotificationDatasource,
        C: GooglePlayDeveloperApiDatasource,
        D: GoogleCloudRtdnNotificationDatasource,
    > IapRepository for IapRepositoryImpl<A, B, C, D>
{
    fn verify_and_get_details(
        &self,
        id: IapId,
        product_type: IapType,
    ) -> Result<IapDetails, GenericServerError> {
        todo!()
    }

    fn parse_apple_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError> {
        todo!()
    }

    fn parse_google_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError> {
        todo!()
    }
}

impl
    IapRepositoryImpl<
        AppStoreServerApiDatasourceImpl,
        AppStoreServerNotificationDatasourceImpl,
        GooglePlayDeveloperApiDatasourceImpl,
        GoogleCloudRtdnNotificationDatasourceImpl,
    >
{
    pub(crate) async fn new(
        apple_api_key: &str,
        apple_key_id: &str,
        apple_issuer_id: &str,
        apple_bundle_id: &str,
        google_play_api_key: &str,
    ) -> Result<Self, GenericServerError> {
        Ok(Self {
            app_store_server_api_datasource: AppStoreServerApiDatasourceImpl::new(
                apple_api_key,
                apple_key_id,
                apple_issuer_id,
                apple_bundle_id,
            )
            .await?,
            app_store_server_notification_datasource: AppStoreServerNotificationDatasourceImpl::new(
            ),
            google_play_developer_api_datasource: GooglePlayDeveloperApiDatasourceImpl::new(
                google_play_api_key,
            )
            .await?,
            google_cloud_rtdn_notification_datasource:
                GoogleCloudRtdnNotificationDatasourceImpl::new(),
        })
    }
}
