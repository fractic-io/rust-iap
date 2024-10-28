use async_trait::async_trait;
use fractic_generic_server_error::{cxt, GenericServerError};

use crate::{
    data::{
        datasources::{
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
        models::{
            app_store_server_api::{self, jws_transaction_decoded_payload_model},
            google_play_developer_api::{
                in_app_product_model, product_purchase_model, subscription_purchase_v2_model,
            },
        },
    },
    domain::{
        entities::{
            iap_details::{
                ConsumableDetails, IapDetails, IapTypeSpecificDetails, MaybeKnown,
                NonConsumableDetails, PriceInfo, SubscriptionDetails,
            },
            iap_product_id::{
                private::_ProductIdType, IapConsumableId, IapNonConsumableId, IapSubscriptionId,
            },
            iap_purchase_id::IapPurchaseId,
            iap_update_notification::IapUpdateNotification,
        },
        repositories::iap_repository::{IapRepository, TypedProductId},
    },
    errors::{
        AppStoreServerApiInvalidResponse, GoogleCloudRtdnNotificationParseError,
        GooglePlayDeveloperApiInvalidResponse,
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
        include_price_info: bool,
    ) -> Result<IapDetails<T::DetailsType>, GenericServerError> {
        cxt!("IapRepositoryImpl::verify_and_get_details");
        match purchase_id {
            IapPurchaseId::AppStoreTransactionId(transaction_id) => {
                let m = self
                    .app_store_server_api_datasource
                    .get_transaction_info(&transaction_id)
                    .await?;
                IapDetails::from_apple_transaction::<T>(m, include_price_info)
            }
            IapPurchaseId::GooglePlayPurchaseToken(token) => {
                let p = if include_price_info {
                    Some(
                        self.google_play_developer_api_datasource
                            .get_in_app_product(&self.application_id, product_id.sku())
                            .await?,
                    )
                } else {
                    None
                };
                match T::product_type() {
                    _ProductIdType::Consumable | _ProductIdType::NonConsumable => {
                        let m = self
                            .google_play_developer_api_datasource
                            .get_product_purchase(&self.application_id, product_id.sku(), &token)
                            .await?;
                        IapDetails::from_google_product_purchase::<T>(m, p)
                    }
                    _ProductIdType::Subscription => {
                        let m = self
                            .google_play_developer_api_datasource
                            .get_subscription_purchase_v2(&self.application_id, &token)
                            .await?;
                        IapDetails::from_google_subscription_purchase::<T>(m, p)
                    }
                }
            }
        }
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

impl<U: IapTypeSpecificDetails> IapDetails<U> {
    fn from_apple_transaction<T: TypedProductId<DetailsType = U>>(
        m: jws_transaction_decoded_payload_model::JwsTransactionDecodedPayloadModel,
        include_price_info: bool,
    ) -> Result<Self, GenericServerError> {
        cxt!("IapDetails::from_apple_transaction");
        Ok(IapDetails {
            is_active: m.revocation_date.is_none() && m.revocation_reason.is_none(),
            is_sandbox: m.environment == app_store_server_api::common::Environment::Sandbox,
            is_finalized_by_client: MaybeKnown::Unknown,
            purchase_time: m.purchase_date,
            region_iso3166_alpha_3: m.storefront.clone(), // Already in ISO 3166-1 alpha-3 format.
            price_info: if include_price_info {
                Some(PriceInfo {
                    price_micros: m.price.ok_or_else(|| {
                        AppStoreServerApiInvalidResponse::new(
                            CXT,
                            "Transaction did not contain price info.",
                        )
                    })? * 1000,
                    currency_iso_4217: m.currency.clone().ok_or_else(|| {
                        AppStoreServerApiInvalidResponse::new(
                            CXT,
                            "Transaction did not contain currency info.",
                        )
                    })?, // Already in ISO 4217 format.
                })
            } else {
                None
            },
            type_specific_details: T::extract_details_from_apple_transaction(&m)?,
        })
    }

    fn from_google_product_purchase<T: TypedProductId<DetailsType = U>>(
        m: product_purchase_model::ProductPurchaseModel,
        p: Option<in_app_product_model::InAppProductModel>,
    ) -> Result<Self, GenericServerError> {
        cxt!("IapDetails::from_google_product_purchase");
        Ok(IapDetails {
            is_active: m.purchase_state == product_purchase_model::PurchaseState::Purchased,
            is_sandbox: m.purchase_type == Some(product_purchase_model::PurchaseType::Test),
            is_finalized_by_client: MaybeKnown::Known(
                m.acknowledgement_state
                    == product_purchase_model::AcknowledgementState::Acknowledged,
            ),
            purchase_time: m.purchase_time_millis,
            region_iso3166_alpha_3: rust_iso3166::from_alpha2(&m.region_code)
                .ok_or_else(|| {
                    GooglePlayDeveloperApiInvalidResponse::with_debug(
                        CXT,
                        "Invalid region code.",
                        m.region_code.clone(),
                    )
                })?
                .alpha3
                .to_string(),
            price_info: p
                .as_ref()
                .map(|p| PriceInfo::from_google_in_app_product_model(p, &m.region_code))
                .transpose()?,
            type_specific_details: T::extract_details_from_google_product_purchase(&m)?,
        })
    }

    fn from_google_subscription_purchase<T: TypedProductId<DetailsType = U>>(
        m: subscription_purchase_v2_model::SubscriptionPurchaseV2Model,
        p: Option<in_app_product_model::InAppProductModel>,
    ) -> Result<Self, GenericServerError> {
        cxt!("IapDetails::from_google_subscription_purchase");
        Ok(IapDetails {
            is_active: (
                m.subscription_state == subscription_purchase_v2_model::SubscriptionState::SubscriptionStateActive ||
                m.subscription_state == subscription_purchase_v2_model::SubscriptionState::SubscriptionStateInGracePeriod
            ),
            is_sandbox: m.test_purchase.is_some(),
            is_finalized_by_client: match m.acknowledgement_state {
                subscription_purchase_v2_model::AcknowledgementState::AcknowledgementStateAcknowledged
                    => MaybeKnown::Known(true),
                subscription_purchase_v2_model::AcknowledgementState::AcknowledgementStatePending
                    => MaybeKnown::Known(false),
                subscription_purchase_v2_model::AcknowledgementState::Unknown(_) |
                subscription_purchase_v2_model::AcknowledgementState::AcknowledgementStateUnspecified
                    => MaybeKnown::Unknown,
            },
            purchase_time: m.start_time.ok_or_else(|| {
                GooglePlayDeveloperApiInvalidResponse::new(
                    CXT,
                    "Subscription did not have a start time.",
                )
            })?,
            region_iso3166_alpha_3: rust_iso3166::from_alpha2(&m.region_code)
                .ok_or_else(|| {
                    GooglePlayDeveloperApiInvalidResponse::with_debug(
                        CXT,
                        "Invalid region code.",
                        m.region_code.clone(),
                    )
                })?
                .alpha3
                .to_string(),
            price_info: p
                .as_ref()
                .map(|p| PriceInfo::from_google_in_app_product_model(p, &m.region_code))
                .transpose()?,
            type_specific_details: T::extract_details_from_google_subscription_purchase(&m)?,
        })
    }
}

impl PriceInfo {
    fn from_google_in_app_product_model(
        p: &in_app_product_model::InAppProductModel,
        region_code: &str,
    ) -> Result<Self, GenericServerError> {
        cxt!("PriceInfo::from_google_in_app_product_model");
        let details = p.prices.get(region_code).ok_or_else(|| {
            GooglePlayDeveloperApiInvalidResponse::with_debug(
                CXT,
                "Region code not found in product prices.",
                region_code.to_string(),
            )
        })?;
        Ok(Self {
            price_micros: details.price_micros.parse::<i64>().map_err(|e| {
                GooglePlayDeveloperApiInvalidResponse::with_debug(
                    CXT,
                    "Price micros could not be parsed.",
                    e.to_string(),
                )
            })?,
            currency_iso_4217: details.currency.clone(),
        })
    }
}

impl TypedProductId for IapNonConsumableId {
    type DetailsType = NonConsumableDetails;

    fn extract_details_from_apple_transaction(
        _m: &jws_transaction_decoded_payload_model::JwsTransactionDecodedPayloadModel,
    ) -> Result<Self::DetailsType, GenericServerError> {
        Ok(NonConsumableDetails {})
    }

    fn extract_details_from_google_product_purchase(
        _m: &product_purchase_model::ProductPurchaseModel,
    ) -> Result<Self::DetailsType, GenericServerError> {
        Ok(NonConsumableDetails {})
    }

    fn extract_details_from_google_subscription_purchase(
        _m: &subscription_purchase_v2_model::SubscriptionPurchaseV2Model,
    ) -> Result<Self::DetailsType, GenericServerError> {
        Ok(NonConsumableDetails {})
    }
}

impl TypedProductId for IapConsumableId {
    type DetailsType = ConsumableDetails;

    fn extract_details_from_apple_transaction(
        m: &jws_transaction_decoded_payload_model::JwsTransactionDecodedPayloadModel,
    ) -> Result<Self::DetailsType, GenericServerError> {
        Ok(ConsumableDetails {
            is_consumed: MaybeKnown::Unknown,
            quantity: m.quantity.map(|q| q as i64).unwrap_or(1),
        })
    }

    fn extract_details_from_google_product_purchase(
        m: &product_purchase_model::ProductPurchaseModel,
    ) -> Result<Self::DetailsType, GenericServerError> {
        Ok(ConsumableDetails {
            is_consumed: MaybeKnown::Known(
                m.consumption_state == product_purchase_model::ConsumptionState::Consumed,
            ),
            quantity: m.quantity.map(|q| q as i64).unwrap_or(1),
        })
    }

    fn extract_details_from_google_subscription_purchase(
        _m: &subscription_purchase_v2_model::SubscriptionPurchaseV2Model,
    ) -> Result<Self::DetailsType, GenericServerError> {
        unimplemented!()
    }
}

impl TypedProductId for IapSubscriptionId {
    type DetailsType = SubscriptionDetails;

    fn extract_details_from_apple_transaction(
        m: &jws_transaction_decoded_payload_model::JwsTransactionDecodedPayloadModel,
    ) -> Result<Self::DetailsType, GenericServerError> {
        cxt!("IapSubscriptionId::extract_details_from_apple_transaction");
        Ok(SubscriptionDetails {
            expiration_time: m.expires_date.ok_or_else(|| {
                AppStoreServerApiInvalidResponse::new(
                    CXT,
                    "Subscription's transaction info did not contain expiration date.",
                )
            })?,
        })
    }

    fn extract_details_from_google_product_purchase(
        _m: &product_purchase_model::ProductPurchaseModel,
    ) -> Result<Self::DetailsType, GenericServerError> {
        unimplemented!()
    }

    fn extract_details_from_google_subscription_purchase(
        m: &subscription_purchase_v2_model::SubscriptionPurchaseV2Model,
    ) -> Result<Self::DetailsType, GenericServerError> {
        cxt!("IapSubscriptionId::extract_details_from_google_subscription_purchase");
        Ok(SubscriptionDetails {
            expiration_time: m
                .line_items
                .iter()
                .max_by_key(|li| li.expiry_time)
                .ok_or_else(|| {
                    GooglePlayDeveloperApiInvalidResponse::new(
                        CXT,
                        "Subscription did not have any line items.",
                    )
                })?
                .expiry_time,
        })
    }
}
