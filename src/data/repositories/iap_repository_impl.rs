use async_trait::async_trait;
use fractic_server_error::ServerError;

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
            app_store_server_api::{self, jws_transaction_decoded_payload_model as at},
            app_store_server_notifications::response_body_v2_decoded_payload_model as an,
            google_cloud_rtdn_notifications::developer_notification_model as gn,
            google_play_developer_api::{
                in_app_product_model as gi, product_purchase_model as gp,
                subscription_purchase_v2_model as gs,
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
                private::{IapProductId, _ProductIdType},
                IapConsumableId, IapNonConsumableId, IapSubscriptionId,
            },
            iap_purchase_id::IapPurchaseId,
            iap_update_notification::{
                IapUpdateNotification, NotificationDetails, SubscriptionEndReason,
            },
        },
        repositories::iap_repository::{IapRepository, TypedProductId},
    },
    errors::{
        AppStoreServerApiInvalidResponse, GoogleCloudRtdnNotificationParseError,
        GooglePlayDeveloperApiInvalidResponse, NotActive,
    },
};

use MaybeKnown::*;

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
    ) -> Result<IapDetails<T::DetailsType>, ServerError> {
        let iap_details = match &purchase_id {
            IapPurchaseId::AppStoreTransactionId(transaction_id) => {
                let m = self
                    .app_store_server_api_datasource
                    .get_transaction_info(&transaction_id)
                    .await?;
                IapDetails::from_apple_transaction::<T>(m, include_price_info)?
            }
            IapPurchaseId::GooglePlayPurchaseToken(token) => {
                match T::product_type() {
                    _ProductIdType::Consumable | _ProductIdType::NonConsumable => {
                        let m = self
                            .google_play_developer_api_datasource
                            .get_product_purchase(&self.application_id, product_id.sku(), token)
                            .await?;
                        let p = if include_price_info {
                            Some(
                                self.google_play_developer_api_datasource
                                    .get_in_app_product(&self.application_id, product_id.sku())
                                    .await?,
                            )
                        } else {
                            None
                        };
                        IapDetails::from_google_product_purchase::<T>(purchase_id, m, p)?
                    }
                    _ProductIdType::Subscription => {
                        let m = self
                            .google_play_developer_api_datasource
                            .get_subscription_purchase_v2(&self.application_id, token)
                            .await?;
                        // Price info not available for subscriptions.
                        //
                        // This would technically be possible with the
                        // monetization.subscriptions API, but would be quite
                        // complex as it requires determining which base plan is
                        // purchased.
                        let p = None;
                        IapDetails::from_google_subscription_purchase::<T>(purchase_id, m, p)?
                    }
                }
            }
        };
        if !iap_details.is_active {
            return Err(NotActive::new());
        }
        Ok(iap_details)
    }

    async fn consume(
        &self,
        product_id: IapConsumableId,
        purchase_id: IapPurchaseId,
    ) -> Result<(), ServerError> {
        match purchase_id {
            IapPurchaseId::GooglePlayPurchaseToken(token) => {
                self.google_play_developer_api_datasource
                    .consume_product_purchase(&self.application_id, product_id.sku(), &token)
                    .await
            }
            _ => Ok(()),
        }
    }

    async fn parse_apple_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, ServerError> {
        let (notification, transaction_info, _subscription_renewal_info) = self
            .app_store_server_notification_datasource
            .parse_notification(body)
            .await?;
        Ok(IapUpdateNotification {
            notification_id: notification.notification_uuid.clone(),
            time: notification.signed_date.clone(),
            details: NotificationDetails::from_apple_notification(notification, transaction_info)?,
        })
    }

    async fn parse_google_notification(
        &self,
        authorization_header: &str,
        body: &str,
    ) -> Result<IapUpdateNotification, ServerError> {
        let (wrapper, notification) = self
            .google_cloud_rtdn_notification_datasource
            .parse_notification(authorization_header, body)
            .await?;
        let application_id = notification.package_name.clone();
        let details = if let Some(_) = notification.test_notification {
            NotificationDetails::Test
        } else if let Some(subscription_notification) = notification.subscription_notification {
            NotificationDetails::from_google_subscription_notification(
                subscription_notification,
                application_id,
                &self.google_play_developer_api_datasource,
            )
            .await?
        } else if let Some(voided_purchase_notification) = notification.voided_purchase_notification
        {
            NotificationDetails::from_google_voided_purchase_notification(
                voided_purchase_notification,
                application_id,
                &self.google_play_developer_api_datasource,
            )
            .await?
        } else if let Some(_) = notification.one_time_product_notification {
            NotificationDetails::Other
        } else {
            return Err(GoogleCloudRtdnNotificationParseError::new(
                "notification did not have one of the recognized types (subscription, one-time purchase, voided purchase, or test)",
            ));
        };
        Ok(IapUpdateNotification {
            notification_id: wrapper.message.message_id,
            time: notification.event_time_millis,
            details,
        })
    }

    async fn request_apple_test_notification(&self, sandbox: bool) -> Result<String, ServerError> {
        self.app_store_server_api_datasource
            .request_test_notification(sandbox)
            .await
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
        application_id: impl Into<String>,
        expected_aud: impl Into<String>,
        apple_api_key: &str,
        apple_key_id: &str,
        apple_issuer_id: &str,
        google_api_key: &str,
    ) -> Result<Self, ServerError> {
        let application_id = application_id.into();
        let expected_aud = expected_aud.into();
        Ok(Self {
            app_store_server_api_datasource: AppStoreServerApiDatasourceImpl::new(
                apple_api_key,
                apple_key_id,
                apple_issuer_id,
                &application_id,
                expected_aud.clone(),
            )
            .await?,
            app_store_server_notification_datasource: AppStoreServerNotificationDatasourceImpl::new(
                expected_aud.clone(),
            ),
            google_play_developer_api_datasource: GooglePlayDeveloperApiDatasourceImpl::new(
                google_api_key,
            )
            .await?,
            google_cloud_rtdn_notification_datasource:
                GoogleCloudRtdnNotificationDatasourceImpl::new(expected_aud),
            application_id,
        })
    }
}

impl<U: IapTypeSpecificDetails> IapDetails<U> {
    fn from_apple_transaction<T: TypedProductId<DetailsType = U>>(
        m: at::JwsTransactionDecodedPayloadModel,
        include_price_info: bool,
    ) -> Result<Self, ServerError> {
        Ok(IapDetails {
            cannonical_id: IapPurchaseId::AppStoreTransactionId(m.original_transaction_id.clone()),
            // NOTE: For subscriptions, we should also check the expiry date.
            // This field is only present for subscriptions, so assume true if
            // it is not present (its presence for subscriptions is validated by
            // subscription-specific parsing logic later on).
            is_active: m.revocation_date.is_none()
                && m.revocation_reason.is_none()
                && m.expires_date
                    .map(|expiry| expiry > chrono::Utc::now())
                    .unwrap_or(true),
            is_sandbox: m.environment == app_store_server_api::common::Environment::Sandbox,
            is_finalized_by_client: Unknown,
            purchase_time: m.purchase_date,
            region_iso3166_alpha_3: m.storefront.clone(), // Already in ISO 3166-1 alpha-3 format.
            price_info: if include_price_info {
                Some(PriceInfo {
                    price_micros: m.price.ok_or_else(|| {
                        AppStoreServerApiInvalidResponse::new(
                            "transaction did not contain price info",
                        )
                    })? * 1000,
                    currency_iso_4217: m.currency.clone().ok_or_else(|| {
                        AppStoreServerApiInvalidResponse::new(
                            "transaction did not contain currency info",
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
        purchase_id: IapPurchaseId,
        m: gp::ProductPurchaseModel,
        p: Option<gi::InAppProductModel>,
    ) -> Result<Self, ServerError> {
        Ok(IapDetails {
            cannonical_id: purchase_id,
            is_active: m.purchase_state == gp::PurchaseState::Purchased,
            is_sandbox: m.purchase_type == Some(gp::PurchaseType::Test),
            is_finalized_by_client: Known(
                m.acknowledgement_state == gp::AcknowledgementState::Acknowledged,
            ),
            purchase_time: m.purchase_time_millis,
            region_iso3166_alpha_3: rust_iso3166::from_alpha2(&m.region_code)
                .ok_or_else(|| {
                    GooglePlayDeveloperApiInvalidResponse::new(&format!(
                        "invalid region code '{}'",
                        m.region_code.clone()
                    ))
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
        purchase_id: IapPurchaseId,
        m: gs::SubscriptionPurchaseV2Model,
        p: Option<gi::InAppProductModel>,
    ) -> Result<Self, ServerError> {
        Ok(IapDetails {
            cannonical_id: purchase_id,
            // NOTE: Certain states (ex. SubscriptionStateCanceled) may indicate
            // the subscription is no longer being renewed, but it may still be
            // active if it has not yet expired.
            is_active: (m.subscription_state == gs::SubscriptionState::SubscriptionStateActive
                || m.subscription_state == gs::SubscriptionState::SubscriptionStatePaused
                || m.subscription_state == gs::SubscriptionState::SubscriptionStateOnHold
                || m.subscription_state == gs::SubscriptionState::SubscriptionStateCanceled
                || m.subscription_state == gs::SubscriptionState::SubscriptionStateInGracePeriod)
                && m.line_items
                    .iter()
                    .any(|li| li.expiry_time > chrono::Utc::now()),
            is_sandbox: m.test_purchase.is_some(),
            is_finalized_by_client: match m.acknowledgement_state {
                gs::AcknowledgementState::AcknowledgementStateAcknowledged => Known(true),
                gs::AcknowledgementState::AcknowledgementStatePending => Known(false),
                gs::AcknowledgementState::Unknown(_)
                | gs::AcknowledgementState::AcknowledgementStateUnspecified => Unknown,
            },
            purchase_time: m.start_time.ok_or_else(|| {
                GooglePlayDeveloperApiInvalidResponse::new("subscription did not have a start time")
            })?,
            region_iso3166_alpha_3: rust_iso3166::from_alpha2(&m.region_code)
                .ok_or_else(|| {
                    GooglePlayDeveloperApiInvalidResponse::new(&format!(
                        "invalid region code '{}'",
                        m.region_code.clone()
                    ))
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
        p: &gi::InAppProductModel,
        region_code: &str,
    ) -> Result<Self, ServerError> {
        let details = p.prices.get(region_code).ok_or_else(|| {
            GooglePlayDeveloperApiInvalidResponse::new(&format!(
                "region code '{}' not found in product prices",
                region_code.to_string()
            ))
        })?;
        Ok(Self {
            price_micros: details.price_micros.parse::<i64>().map_err(|e| {
                GooglePlayDeveloperApiInvalidResponse::with_debug(
                    "price micros could not be parsed",
                    &e,
                )
            })?,
            currency_iso_4217: details.currency.clone(),
        })
    }
}

impl TypedProductId for IapNonConsumableId {
    type DetailsType = NonConsumableDetails;

    fn extract_details_from_apple_transaction(
        _m: &at::JwsTransactionDecodedPayloadModel,
    ) -> Result<Self::DetailsType, ServerError> {
        Ok(NonConsumableDetails {})
    }

    fn extract_details_from_google_product_purchase(
        _m: &gp::ProductPurchaseModel,
    ) -> Result<Self::DetailsType, ServerError> {
        Ok(NonConsumableDetails {})
    }

    fn extract_details_from_google_subscription_purchase(
        _m: &gs::SubscriptionPurchaseV2Model,
    ) -> Result<Self::DetailsType, ServerError> {
        unreachable!()
    }
}

impl TypedProductId for IapConsumableId {
    type DetailsType = ConsumableDetails;

    fn extract_details_from_apple_transaction(
        m: &at::JwsTransactionDecodedPayloadModel,
    ) -> Result<Self::DetailsType, ServerError> {
        Ok(ConsumableDetails {
            is_consumed: Unknown,
            quantity: m.quantity.map(|q| q as i64).unwrap_or(1),
        })
    }

    fn extract_details_from_google_product_purchase(
        m: &gp::ProductPurchaseModel,
    ) -> Result<Self::DetailsType, ServerError> {
        Ok(ConsumableDetails {
            is_consumed: Known(m.consumption_state == gp::ConsumptionState::Consumed),
            quantity: m.quantity.map(|q| q as i64).unwrap_or(1),
        })
    }

    fn extract_details_from_google_subscription_purchase(
        _m: &gs::SubscriptionPurchaseV2Model,
    ) -> Result<Self::DetailsType, ServerError> {
        unreachable!()
    }
}

impl TypedProductId for IapSubscriptionId {
    type DetailsType = SubscriptionDetails;

    fn extract_details_from_apple_transaction(
        m: &at::JwsTransactionDecodedPayloadModel,
    ) -> Result<Self::DetailsType, ServerError> {
        Ok(SubscriptionDetails {
            expiration_time: m.expires_date.ok_or_else(|| {
                AppStoreServerApiInvalidResponse::new(
                    "subscription's transaction info did not contain expiration date",
                )
            })?,
        })
    }

    fn extract_details_from_google_product_purchase(
        _m: &gp::ProductPurchaseModel,
    ) -> Result<Self::DetailsType, ServerError> {
        unreachable!()
    }

    fn extract_details_from_google_subscription_purchase(
        m: &gs::SubscriptionPurchaseV2Model,
    ) -> Result<Self::DetailsType, ServerError> {
        Ok(SubscriptionDetails {
            expiration_time: m
                .line_items
                .iter()
                .max_by_key(|li| li.expiry_time)
                .ok_or_else(|| {
                    GooglePlayDeveloperApiInvalidResponse::new(
                        "subscription did not have any line items",
                    )
                })?
                .expiry_time,
        })
    }
}

impl NotificationDetails {
    fn from_apple_notification(
        notification: an::ResponseBodyV2DecodedPayloadModel,
        transaction_info: Option<at::JwsTransactionDecodedPayloadModel>,
    ) -> Result<Self, ServerError> {
        let expected_data_missing_err = || {
            Err(AppStoreServerApiInvalidResponse::new(&format!(
                "notification type {:?} did not contain expected data",
                notification.notification_type
            )))
        };
        Ok(
            match (&notification.notification_type, &notification.subtype) {
                (an::NotificationType::Test, _) => NotificationDetails::Test,

                (an::NotificationType::Subscribed, _) => {
                    let (Some(data), Some(transaction_info)) =
                        (notification.data, transaction_info)
                    else {
                        return expected_data_missing_err();
                    };
                    NotificationDetails::SubscriptionStarted {
                        application_id: data.bundle_id,
                        product_id: IapSubscriptionId(transaction_info.product_id.clone()),
                        purchase_id: IapPurchaseId::AppStoreTransactionId(
                            transaction_info.original_transaction_id.clone(),
                        ),
                        details: IapDetails::from_apple_transaction::<IapSubscriptionId>(
                            transaction_info,
                            false,
                        )?,
                    }
                }

                (an::NotificationType::DidRenew, _)
                | (
                    an::NotificationType::DidFailToRenew,
                    Some(an::NotificationSubtype::GracePeriod),
                )
                | (an::NotificationType::RefundReversed, _)
                | (an::NotificationType::RenewalExtended, _) => {
                    let (Some(data), Some(transaction_info)) =
                        (notification.data, transaction_info)
                    else {
                        return expected_data_missing_err();
                    };
                    NotificationDetails::SubscriptionExpiryChanged {
                        application_id: data.bundle_id,
                        product_id: IapSubscriptionId(transaction_info.product_id.clone()),
                        purchase_id: IapPurchaseId::AppStoreTransactionId(
                            transaction_info.original_transaction_id.clone(),
                        ),
                        renewal_id: if notification.notification_type
                            == an::NotificationType::DidRenew
                        {
                            Some(transaction_info.transaction_id.clone())
                        } else {
                            None
                        },
                        details: IapDetails::from_apple_transaction::<IapSubscriptionId>(
                            transaction_info,
                            false,
                        )?,
                    }
                }

                (an::NotificationType::DidFailToRenew, _)
                | (an::NotificationType::Expired, _)
                | (an::NotificationType::GracePeriodExpired, _) => {
                    let (Some(data), Some(transaction_info)) =
                        (notification.data, transaction_info)
                    else {
                        return expected_data_missing_err();
                    };
                    NotificationDetails::SubscriptionEnded {
                        application_id: data.bundle_id,
                        product_id: IapSubscriptionId(transaction_info.product_id.clone()),
                        purchase_id: IapPurchaseId::AppStoreTransactionId(
                            transaction_info.original_transaction_id.clone(),
                        ),
                        details: IapDetails::from_apple_transaction::<IapSubscriptionId>(
                            transaction_info,
                            false,
                        )?,
                        reason: if notification.notification_type
                            == an::NotificationType::GracePeriodExpired
                            || notification.subtype == Some(an::NotificationSubtype::BillingRetry)
                        {
                            SubscriptionEndReason::FailedToRenew
                        } else if notification.subtype == Some(an::NotificationSubtype::Voluntary) {
                            SubscriptionEndReason::Cancelled { details: None }
                        } else if notification.subtype
                            == Some(an::NotificationSubtype::PriceIncrease)
                        {
                            SubscriptionEndReason::DeclinedPriceIncrease
                        } else {
                            SubscriptionEndReason::Unknown
                        },
                    }
                }

                (an::NotificationType::Refund, _) | (an::NotificationType::Revoke, _) => {
                    let (Some(data), Some(transaction_info)) =
                        (notification.data, transaction_info)
                    else {
                        return expected_data_missing_err();
                    };
                    match transaction_info.transaction_type {
                        at::TransactionType::NonConsumable => {
                            NotificationDetails::NonConsumableVoided {
                                application_id: data.bundle_id,
                                product_id: IapNonConsumableId(transaction_info.product_id.clone()),
                                purchase_id: IapPurchaseId::AppStoreTransactionId(
                                    transaction_info.original_transaction_id.clone(),
                                ),
                                reason: Some(format!("{:?}", transaction_info.revocation_reason)),
                                details: IapDetails::from_apple_transaction::<IapNonConsumableId>(
                                    transaction_info,
                                    false,
                                )?,
                                is_refunded: notification.notification_type
                                    == an::NotificationType::Refund,
                            }
                        }
                        at::TransactionType::Consumable => NotificationDetails::ConsumableVoided {
                            application_id: data.bundle_id,
                            product_id: IapConsumableId(transaction_info.product_id.clone()),
                            purchase_id: IapPurchaseId::AppStoreTransactionId(
                                transaction_info.original_transaction_id.clone(),
                            ),
                            reason: Some(format!("{:?}", transaction_info.revocation_reason)),
                            details: IapDetails::from_apple_transaction::<IapConsumableId>(
                                transaction_info,
                                false,
                            )?,
                            is_refunded: notification.notification_type
                                == an::NotificationType::Refund,
                        },
                        _ => NotificationDetails::SubscriptionEnded {
                            application_id: data.bundle_id,
                            product_id: IapSubscriptionId(transaction_info.product_id.clone()),
                            purchase_id: IapPurchaseId::AppStoreTransactionId(
                                transaction_info.original_transaction_id.clone(),
                            ),
                            details: IapDetails::from_apple_transaction::<IapSubscriptionId>(
                                transaction_info,
                                false,
                            )?,
                            reason: SubscriptionEndReason::Voided {
                                is_refunded: notification.notification_type
                                    == an::NotificationType::Refund,
                            },
                        },
                    }
                }

                // Changes that do not affect validity or expiry.
                (an::NotificationType::DidChangeRenewalPref, _)
                | (an::NotificationType::DidChangeRenewalStatus, _)
                | (an::NotificationType::OfferRedeemed, _)
                | (an::NotificationType::PriceIncrease, _)
                | (an::NotificationType::RefundDeclined, _)
                | (an::NotificationType::RenewalExtension, _)
                | (an::NotificationType::ExternalPurchaseToken, _)
                | (an::NotificationType::OneTimeCharge, _)
                | (an::NotificationType::ConsumptionRequest, _)
                | (an::NotificationType::Unknown(_), _) => NotificationDetails::Other,
            },
        )
    }

    async fn from_google_subscription_notification<T: GooglePlayDeveloperApiDatasource>(
        notification: gn::SubscriptionNotification,
        application_id: String,
        google_play_developer_api_datasource: &T,
    ) -> Result<Self, ServerError> {
        let api_data = google_play_developer_api_datasource
            .get_subscription_purchase_v2(&application_id, &notification.purchase_token)
            .await?;
        let product_id = IapSubscriptionId(
            api_data
                .line_items
                .last()
                .ok_or_else(|| {
                    GooglePlayDeveloperApiInvalidResponse::new(
                        "subscription did not have any line items",
                    )
                })?
                .product_id
                .clone(),
        );
        let purchase_id = IapPurchaseId::GooglePlayPurchaseToken(notification.purchase_token);
        Ok(match notification.notification_type {
            gn::SubscriptionNotificationType::SubscriptionPurchased => {
                NotificationDetails::SubscriptionStarted {
                    application_id,
                    product_id,
                    purchase_id: purchase_id.clone(),
                    details: IapDetails::from_google_subscription_purchase::<IapSubscriptionId>(
                        purchase_id,
                        api_data,
                        None,
                    )?,
                }
            }

            gn::SubscriptionNotificationType::SubscriptionRenewed
            | gn::SubscriptionNotificationType::SubscriptionRecovered
            | gn::SubscriptionNotificationType::SubscriptionInGracePeriod
            | gn::SubscriptionNotificationType::SubscriptionDeferred => {
                NotificationDetails::SubscriptionExpiryChanged {
                    application_id,
                    product_id,
                    purchase_id: purchase_id.clone(),
                    renewal_id: if notification.notification_type
                        == gn::SubscriptionNotificationType::SubscriptionRenewed
                        || notification.notification_type
                            == gn::SubscriptionNotificationType::SubscriptionRecovered
                    {
                        Some(api_data.latest_order_id.clone())
                    } else {
                        None
                    },
                    details: IapDetails::from_google_subscription_purchase::<IapSubscriptionId>(
                        purchase_id,
                        api_data,
                        None,
                    )?,
                }
            }

            gn::SubscriptionNotificationType::SubscriptionExpired
            | gn::SubscriptionNotificationType::SubscriptionRevoked
            | gn::SubscriptionNotificationType::SubscriptionPaused
            | gn::SubscriptionNotificationType::SubscriptionOnHold => {
                let reason = if notification.notification_type
                    == gn::SubscriptionNotificationType::SubscriptionPaused
                {
                    SubscriptionEndReason::Paused
                } else if api_data
                    .canceled_state_context
                    .as_ref()
                    .map(|csc| csc.system_initiated_cancellation.is_some())
                    .unwrap_or(false)
                {
                    SubscriptionEndReason::FailedToRenew
                } else if api_data
                    .canceled_state_context
                    .as_ref()
                    .map(|csc| csc.user_initiated_cancellation.is_some())
                    .unwrap_or(false)
                {
                    SubscriptionEndReason::Cancelled {
                        details: Some(format!(
                            "{:?}",
                            api_data
                                .canceled_state_context
                                .as_ref()
                                .unwrap()
                                .user_initiated_cancellation
                                .as_ref()
                                .unwrap()
                        )),
                    }
                } else {
                    SubscriptionEndReason::Unknown
                };
                NotificationDetails::SubscriptionEnded {
                    application_id,
                    product_id,
                    purchase_id: purchase_id.clone(),
                    details: IapDetails::from_google_subscription_purchase::<IapSubscriptionId>(
                        purchase_id,
                        api_data,
                        None,
                    )?,
                    reason,
                }
            }

            // Perhaps counterintuitively, subscription cancellation and restart
            // events are not important as they do not affect subscription
            // expiry. After cancellation, the subscription will continue as
            // normal until the expiry date, at which point an expiry
            // notification is received and caught above.
            //
            // To continue the confusing naming, pausing should technically be
            // the same way, but pausing the subscription does not cause a
            // SubscriptionPaused event. Rather, it causes a
            // SubscriptionPauseScheduleChanged event, and the
            // SubscriptionPaused event indicates the start of the actual pause
            // period, which should not be ignored.
            //
            // Note on capturing cancellation reason:
            //   Since we fetch the full subscription information upon receiving
            //   an expiry event, we will be able to see cancellation reason at
            //   that point, so we don't need to capture it now.
            gn::SubscriptionNotificationType::SubscriptionRestarted
            | gn::SubscriptionNotificationType::SubscriptionCanceled => NotificationDetails::Other,

            // Changes that do not affect validity or expiry.
            gn::SubscriptionNotificationType::SubscriptionPriceChangeConfirmed
            | gn::SubscriptionNotificationType::SubscriptionPauseScheduleChanged
            | gn::SubscriptionNotificationType::SubscriptionPendingPurchaseCanceled => {
                NotificationDetails::Other
            }
        })
    }

    async fn from_google_voided_purchase_notification<T: GooglePlayDeveloperApiDatasource>(
        notification: gn::VoidedPurchaseNotification,
        application_id: String,
        google_play_developer_api_datasource: &T,
    ) -> Result<Self, ServerError> {
        Ok(match notification.product_type {
            gn::VoidedPurchaseProductType::ProductTypeOneTime => {
                // Unfortunately, we don't have access to the product ID here,
                // so we have no way to fetch the product details, or to
                // determine if the product is a consumable / non-consumable.
                NotificationDetails::UnknownOneTimePurchaseVoided {
                    application_id,
                    purchase_id: IapPurchaseId::GooglePlayPurchaseToken(
                        notification.purchase_token,
                    ),
                    is_refunded: notification.refund_type
                        == gn::VoidedPurchaseRefundType::RefundTypeFullRefund,
                    reason: None,
                }
            }
            gn::VoidedPurchaseProductType::ProductTypeSubscription => {
                let m = google_play_developer_api_datasource
                    .get_subscription_purchase_v2(&application_id, &notification.purchase_token)
                    .await?;
                let purchase_id =
                    IapPurchaseId::GooglePlayPurchaseToken(notification.purchase_token);
                NotificationDetails::SubscriptionEnded {
                    application_id,
                    product_id: IapSubscriptionId(
                        m.line_items
                            .last()
                            .ok_or_else(|| {
                                GooglePlayDeveloperApiInvalidResponse::new(
                                    "subscription did not have any line items",
                                )
                            })?
                            .product_id
                            .clone(),
                    ),
                    purchase_id: purchase_id.clone(),
                    details: IapDetails::from_google_subscription_purchase::<IapSubscriptionId>(
                        purchase_id,
                        m,
                        None,
                    )?,
                    reason: SubscriptionEndReason::Voided {
                        is_refunded: notification.refund_type
                            == gn::VoidedPurchaseRefundType::RefundTypeFullRefund,
                    },
                }
            }
        })
    }
}
