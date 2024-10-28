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
            app_store_server_notifications::response_body_v2_decoded_payload_model::{
                NotificationSubtype, NotificationType,
            },
            google_cloud_rtdn_notifications::developer_notification_model::{
                SubscriptionNotificationType, VoidedPurchaseProductType, VoidedPurchaseRefundType,
            },
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
            iap_update_notification::{
                IapUpdateNotification, NotificationDetails, SubscriptionEndReason,
            },
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
        cxt!("IapRepositoryImpl::parse_apple_notification");
        let (notification, transaction_info, _subscription_renewal_info) = self
            .app_store_server_notification_datasource
            .parse_notification(body)
            .await?;
        let expected_data_missing_err = || {
            Err(AppStoreServerApiInvalidResponse::with_debug(
                CXT,
                "Notification did not contain expected data.",
                format!("{:?}", notification.notification_type),
            ))
        };
        let details = match (&notification.notification_type, &notification.subtype) {
            (NotificationType::Test, _) => NotificationDetails::Test,

            (NotificationType::Subscribed, _) => {
                let (Some(data), Some(transaction_info)) = (notification.data, transaction_info)
                else {
                    return expected_data_missing_err();
                };
                NotificationDetails::SubscriptionStarted {
                    application_id: data.bundle_id,
                    product_id: IapSubscriptionId(transaction_info.product_id.clone()),
                    purchase_id: IapPurchaseId::AppStoreTransactionId(
                        transaction_info.transaction_id.clone(),
                    ),
                    details: IapDetails::from_apple_transaction::<IapSubscriptionId>(
                        transaction_info,
                        false,
                    )?,
                }
            }

            (NotificationType::DidRenew, _)
            | (NotificationType::DidFailToRenew, Some(NotificationSubtype::GracePeriod))
            | (NotificationType::RefundReversed, _) => {
                let (Some(data), Some(transaction_info)) = (notification.data, transaction_info)
                else {
                    return expected_data_missing_err();
                };
                NotificationDetails::SubscriptionExpiryChanged {
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

            (NotificationType::DidFailToRenew, _)
            | (NotificationType::Expired, _)
            | (NotificationType::GracePeriodExpired, _) => {
                let (Some(data), Some(transaction_info)) = (notification.data, transaction_info)
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
                        == NotificationType::GracePeriodExpired
                        || notification.subtype == Some(NotificationSubtype::BillingRetry)
                    {
                        SubscriptionEndReason::FailedToRenew
                    } else if notification.subtype == Some(NotificationSubtype::Voluntary) {
                        SubscriptionEndReason::Cancelled { reason: None }
                    } else if notification.subtype == Some(NotificationSubtype::PriceIncrease) {
                        SubscriptionEndReason::DeclinedPriceIncrease
                    } else {
                        SubscriptionEndReason::Unknown
                    },
                }
            }

            (NotificationType::Refund, _) | (NotificationType::Revoke, _) => {
                let (Some(data), Some(transaction_info)) = (notification.data, transaction_info)
                else {
                    return expected_data_missing_err();
                };
                match transaction_info.transaction_type {
                    jws_transaction_decoded_payload_model::TransactionType::NonConsumable => {
                        NotificationDetails::NonConsumableVoided {
                            application_id: data.bundle_id,
                            product_id: IapNonConsumableId(transaction_info.product_id.clone()),
                            purchase_id: IapPurchaseId::AppStoreTransactionId(
                                transaction_info.transaction_id.clone(),
                            ),
                            reason: Some(format!("{:?}", transaction_info.revocation_reason)),
                            details: IapDetails::from_apple_transaction::<IapNonConsumableId>(
                                transaction_info,
                                false,
                            )?,
                            is_refunded: notification.notification_type == NotificationType::Refund,
                        }
                    }
                    jws_transaction_decoded_payload_model::TransactionType::Consumable => {
                        NotificationDetails::ConsumableVoided {
                            application_id: data.bundle_id,
                            product_id: IapConsumableId(transaction_info.product_id.clone()),
                            purchase_id: IapPurchaseId::AppStoreTransactionId(
                                transaction_info.transaction_id.clone(),
                            ),
                            reason: Some(format!("{:?}", transaction_info.revocation_reason)),
                            details: IapDetails::from_apple_transaction::<IapConsumableId>(
                                transaction_info,
                                false,
                            )?,
                            is_refunded: notification.notification_type == NotificationType::Refund,
                        }
                    }
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
                            is_refunded: notification.notification_type == NotificationType::Refund,
                        },
                    },
                }
            }

            (NotificationType::DidChangeRenewalPref, _)
            | (NotificationType::DidChangeRenewalStatus, _)
            | (NotificationType::OfferRedeemed, _)
            | (NotificationType::PriceIncrease, _)
            | (NotificationType::RefundDeclined, _)
            | (NotificationType::RenewalExtended, _)
            | (NotificationType::RenewalExtension, _)
            | (NotificationType::ExternalPurchaseToken, _)
            | (NotificationType::OneTimeCharge, _)
            | (NotificationType::Unknown(_), _) => NotificationDetails::Other,
        };
        Ok(IapUpdateNotification {
            notification_id: notification.notification_uuid,
            time: notification.signed_date,
            details,
        })
    }

    async fn parse_google_notification(
        &self,
        body: &str,
    ) -> Result<IapUpdateNotification, GenericServerError> {
        cxt!("IapRepositoryImpl::parse_google_notification");
        let (wrapper, notification) = self
            .google_cloud_rtdn_notification_datasource
            .parse_notification(body)
            .await?;
        let details = if let Some(_) = notification.test_notification {
            NotificationDetails::Test
        } else if let Some(n) = notification.subscription_notification {
            let m = self
                .google_play_developer_api_datasource
                .get_subscription_purchase_v2(&notification.package_name, &n.purchase_token)
                .await?;
            let application_id = notification.package_name;
            let product_id = IapSubscriptionId(
                m.line_items
                    .last()
                    .ok_or_else(|| {
                        GooglePlayDeveloperApiInvalidResponse::new(
                            CXT,
                            "Subscription did not have any line items.",
                        )
                    })?
                    .product_id
                    .clone(),
            );
            let purchase_id = IapPurchaseId::GooglePlayPurchaseToken(n.purchase_token);
            match n.notification_type {
                SubscriptionNotificationType::SubscriptionPurchased => {
                    NotificationDetails::SubscriptionStarted {
                        application_id,
                        product_id,
                        purchase_id,
                        details: IapDetails::from_google_subscription_purchase::<IapSubscriptionId>(
                            m, None,
                        )?,
                    }
                }
                SubscriptionNotificationType::SubscriptionRecovered
                | SubscriptionNotificationType::SubscriptionRenewed
                | SubscriptionNotificationType::SubscriptionInGracePeriod
                | SubscriptionNotificationType::SubscriptionDeferred => {
                    NotificationDetails::SubscriptionExpiryChanged {
                        application_id,
                        product_id,
                        purchase_id,
                        details: IapDetails::from_google_subscription_purchase::<IapSubscriptionId>(
                            m, None,
                        )?,
                    }
                }
                SubscriptionNotificationType::SubscriptionOnHold
                | SubscriptionNotificationType::SubscriptionPaused
                | SubscriptionNotificationType::SubscriptionExpired
                | SubscriptionNotificationType::SubscriptionRevoked => {
                    NotificationDetails::SubscriptionEnded {
                        application_id,
                        product_id,
                        purchase_id,
                        reason: SubscriptionEndReason::Cancelled {
                            reason: Some(format!("{:?}", m.canceled_state_context)),
                        },
                        details: IapDetails::from_google_subscription_purchase::<IapSubscriptionId>(
                            m, None,
                        )?,
                    }
                }
                SubscriptionNotificationType::SubscriptionRestarted // Unrelated to expiry.
                | SubscriptionNotificationType::SubscriptionCanceled // Unrelated to expiry.
                | SubscriptionNotificationType::SubscriptionPriceChangeConfirmed
                | SubscriptionNotificationType::SubscriptionPauseScheduleChanged
                | SubscriptionNotificationType::SubscriptionPendingPurchaseCanceled => {
                    NotificationDetails::Other
                }
            }
        } else if let Some(n) = notification.voided_purchase_notification {
            match n.product_type {
                VoidedPurchaseProductType::ProductTypeOneTime => {
                    NotificationDetails::UnknownOneTimePurchaseVoided {
                        application_id: notification.package_name,
                        purchase_id: IapPurchaseId::GooglePlayPurchaseToken(n.purchase_token),
                        is_refunded: n.refund_type
                            == VoidedPurchaseRefundType::RefundTypeFullRefund,
                        reason: None,
                    }
                }
                VoidedPurchaseProductType::ProductTypeSubscription => {
                    let m = self
                        .google_play_developer_api_datasource
                        .get_subscription_purchase_v2(&notification.package_name, &n.purchase_token)
                        .await?;
                    NotificationDetails::SubscriptionEnded {
                        application_id: notification.package_name,
                        product_id: IapSubscriptionId(
                            m.line_items
                                .last()
                                .ok_or_else(|| {
                                    GooglePlayDeveloperApiInvalidResponse::new(
                                        CXT,
                                        "Subscription did not have any line items.",
                                    )
                                })?
                                .product_id
                                .clone(),
                        ),
                        purchase_id: IapPurchaseId::GooglePlayPurchaseToken(n.purchase_token),
                        details: IapDetails::from_google_subscription_purchase::<IapSubscriptionId>(
                            m, None,
                        )?,
                        reason: SubscriptionEndReason::Voided {
                            is_refunded: n.refund_type
                                == VoidedPurchaseRefundType::RefundTypeFullRefund,
                        },
                    }
                }
            }
        } else if let Some(_) = notification.one_time_product_notification {
            NotificationDetails::Other
        } else {
            return Err(GoogleCloudRtdnNotificationParseError::new(
                CXT,
                "Notification did not have one of the recognized types (subscription, one-time purchase, voided purchase, or test).",
            ));
        };
        Ok(IapUpdateNotification {
            notification_id: wrapper.message.message_id,
            time: notification.event_time_millis,
            details,
        })
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
