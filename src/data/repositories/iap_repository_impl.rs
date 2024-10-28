use async_trait::async_trait;
use fractic_generic_server_error::{cxt, GenericServerError};

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
            iap_details::IapDetails, iap_product_id::private::_ProductIdType,
            iap_purchase_id::IapPurchaseId, iap_update_notification::IapUpdateNotification,
        },
        repositories::iap_repository::{IapRepository, TypedProductId},
    },
    errors::{GoogleCloudRtdnNotificationParseError, GooglePlayDeveloperApiInvalidResponse},
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
    application_id: String,
}

#[async_trait]
impl<
        A: AppStoreServerApiDatasource,
        B: AppStoreServerNotificationDatasource,
        C: GooglePlayDeveloperApiDatasource,
        D: GoogleCloudRtdnNotificationDatasource,
    > IapRepository for IapRepositoryImpl<A, B, C, D>
{
    async fn verify_and_get_details<T: TypedProductId>(
        &self,
        product_id: T,
        purchase_id: IapPurchaseId,
    ) -> Result<IapDetails<T::DetailsType>, GenericServerError> {
        cxt!("IapRepositoryImpl::verify_and_get_details");
        Ok(match purchase_id {
            IapPurchaseId::AppStoreTransactionId(transaction_id) => {
                let apple_transaction_info = self
                    .app_store_server_api_datasource
                    .get_transaction_info(&transaction_id)
                    .await?;
                IapDetails {
                    is_active: true,
                    purchase_time: apple_transaction_info.purchase_date,
                    type_specific_details: T::extract_details_from_apple_transaction(
                        &apple_transaction_info,
                    ),
                }
            }
            IapPurchaseId::GooglePlayPurchaseToken(token) => match T::product_type() {
                _ProductIdType::Consumable | _ProductIdType::NonConsumable => {
                    let google_product_purchase = self
                        .google_play_developer_api_datasource
                        .get_product_purchase(&self.application_id, product_id.sku(), &token)
                        .await?;
                    IapDetails {
                        is_active: true,
                        purchase_time: google_product_purchase.purchase_time_millis,
                        type_specific_details: T::extract_details_from_google_product_purchase(
                            &google_product_purchase,
                        ),
                    }
                }
                _ProductIdType::Subscription => {
                    let google_subscription_purchase = self
                        .google_play_developer_api_datasource
                        .get_subscription_purchase_v2(&self.application_id, &token)
                        .await?;
                    IapDetails {
                        is_active: true,
                        purchase_time: google_subscription_purchase.start_time.ok_or_else(
                            || {
                                GooglePlayDeveloperApiInvalidResponse::new(
                                    CXT,
                                    "Subscription did not have a start time.",
                                )
                            },
                        )?,
                        type_specific_details: T::extract_details_from_google_subscription_purchase(
                            &google_subscription_purchase,
                        ),
                    }
                }
            },
        })
    }

    async fn parse_apple_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError> {
        let (_notification, transaction_info, _subscription_renewal_info) = self
            .app_store_server_notification_datasource
            .parse_notification(body)
            .await?;
        Ok(IapUpdateNotification::TestConsumable {
            purchase_id: transaction_info
                .map(|t| IapPurchaseId::AppStoreTransactionId(t.transaction_id)),
            details: None,
        })
    }

    async fn parse_google_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError> {
        cxt!("IapRepositoryImpl::parse_google_notification");
        let notification = self
            .google_cloud_rtdn_notification_datasource
            .parse_notification(body)
            .await?;
        if let Some(n) = notification.subscription_notification {
            Ok(IapUpdateNotification::TestConsumable {
                purchase_id: Some(IapPurchaseId::GooglePlayPurchaseToken(n.purchase_token)),
                details: None,
            })
        } else if let Some(n) = notification.one_time_product_notification {
            Ok(IapUpdateNotification::TestConsumable {
                purchase_id: Some(IapPurchaseId::GooglePlayPurchaseToken(n.purchase_token)),
                details: None,
            })
        } else if let Some(n) = notification.voided_purchase_notification {
            Ok(IapUpdateNotification::TestConsumable {
                purchase_id: Some(IapPurchaseId::GooglePlayPurchaseToken(n.purchase_token)),
                details: None,
            })
        } else {
            Err(GoogleCloudRtdnNotificationParseError::new(
                CXT,
                "Notification did not have one of the recognized types (subscription, one-time purchase, voided purchase, or test).",
            )
            .into())
        }
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
        application_id: String,
        apple_api_key: &str,
        apple_key_id: &str,
        apple_issuer_id: &str,
        google_play_api_key: &str,
    ) -> Result<Self, GenericServerError> {
        Ok(Self {
            app_store_server_api_datasource: AppStoreServerApiDatasourceImpl::new(
                apple_api_key,
                apple_key_id,
                apple_issuer_id,
                &application_id,
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
            application_id,
        })
    }
}
