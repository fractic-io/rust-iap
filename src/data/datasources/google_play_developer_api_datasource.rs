use std::any::TypeId;

use async_trait::async_trait;
use fractic_server_error::ServerError;
use reqwest::header::{AUTHORIZATION, CONTENT_LENGTH};
use serde::de::DeserializeOwned;
use yup_oauth2::{parse_service_account_key, ServiceAccountAuthenticator};

use crate::{
    data::models::google_play_developer_api::{
        in_app_product_model::InAppProductModel, product_purchase_model::ProductPurchaseModel,
        subscription_purchase_v2_model::SubscriptionPurchaseV2Model,
    },
    errors::{GooglePlayDeveloperApiError, GooglePlayDeveloperApiKeyInvalid},
};

#[derive(Debug, Clone, Copy)]
enum Method {
    Post,
    Get,
}

#[async_trait]
pub(crate) trait GooglePlayDeveloperApiDatasource: Send + Sync {
    /// purchases.products.get:
    /// https://developers.google.com/android-publisher/api-ref/rest/v3/purchases.products/get
    ///
    /// packageName:
    ///   The package name of the application the inapp product was sold in (for
    ///   example, 'com.some.thing').
    /// productId:
    ///   The inapp product SKU (for example, 'com.some.thing.inapp1').
    /// token:
    ///   The token provided to the user's device when the inapp product was
    ///   purchased.
    async fn get_product_purchase(
        &self,
        package_name: &str,
        product_id: &str,
        token: &str,
    ) -> Result<ProductPurchaseModel, ServerError>;

    /// purchases.subscriptionsv2.get:
    /// https://developers.google.com/android-publisher/api-ref/rest/v3/purchases.subscriptionsv2/get
    ///
    /// packageName:
    ///   The package of the application for which this subscription was
    ///   purchased (for example, 'com.some.thing').
    /// token:
    ///   The token provided to the user's device when the subscription was
    ///   purchased.
    async fn get_subscription_purchase_v2(
        &self,
        package_name: &str,
        token: &str,
    ) -> Result<SubscriptionPurchaseV2Model, ServerError>;

    /// inappproducts.get:
    /// https://developers.google.com/android-publisher/api-ref/rest/v3/inappproducts/get
    ///
    /// packageName:
    ///   Package name of the app.
    /// sku:
    ///   Unique identifier for the in-app product.
    async fn get_in_app_product(
        &self,
        package_name: &str,
        sku: &str,
    ) -> Result<InAppProductModel, ServerError>;

    /// purchases.products.consume:
    /// https://developers.google.com/android-publisher/api-ref/rest/v3/purchases.products/consume
    ///
    /// packageName:
    ///   The package name of the application the inapp product was sold in (for
    ///   example, 'com.some.thing').
    /// productId:
    ///   The inapp product SKU (for example, 'com.some.thing.inapp1').
    /// token:
    ///   The token provided to the user's device when the inapp product was
    ///   purchased.
    async fn consume_product_purchase(
        &self,
        package_name: &str,
        product_id: &str,
        token: &str,
    ) -> Result<(), ServerError>;
}

pub(crate) struct GooglePlayDeveloperApiDatasourceImpl {
    access_token: String,
}

#[async_trait]
impl GooglePlayDeveloperApiDatasource for GooglePlayDeveloperApiDatasourceImpl {
    async fn get_product_purchase(
        &self,
        package_name: &str,
        product_id: &str,
        token: &str,
    ) -> Result<ProductPurchaseModel, ServerError> {
        let url = format!("https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{package_name}/purchases/products/{product_id}/tokens/{token}");
        self.callout(&url, "purchases.products.get", Method::Get)
            .await
    }

    async fn get_subscription_purchase_v2(
        &self,
        package_name: &str,
        token: &str,
    ) -> Result<SubscriptionPurchaseV2Model, ServerError> {
        let url = format!("https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{package_name}/purchases/subscriptionsv2/tokens/{token}");
        self.callout(&url, "purchases.subscriptionsv2.get", Method::Get)
            .await
    }

    async fn get_in_app_product(
        &self,
        package_name: &str,
        sku: &str,
    ) -> Result<InAppProductModel, ServerError> {
        let url = format!("https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{package_name}/inappproducts/{sku}");
        self.callout(&url, "inappproducts.get", Method::Get).await
    }

    async fn consume_product_purchase(
        &self,
        package_name: &str,
        product_id: &str,
        token: &str,
    ) -> Result<(), ServerError> {
        let url = format!("https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{package_name}/purchases/products/{product_id}/tokens/{token}:consume");
        self.callout(&url, "purchases.products.consume", Method::Post)
            .await
    }
}

impl GooglePlayDeveloperApiDatasourceImpl {
    pub(crate) async fn new(api_key: &str) -> Result<Self, ServerError> {
        Ok(Self {
            access_token: Self::build_access_token(api_key).await?,
        })
    }

    async fn build_access_token(api_key: &str) -> Result<String, ServerError> {
        let key = parse_service_account_key(api_key).map_err(|e| {
            GooglePlayDeveloperApiKeyInvalid::with_debug(
                "Google Play API key could not be parsed",
                &e,
            )
        })?;
        let authenticator = ServiceAccountAuthenticator::builder(key)
            .build()
            .await
            .map_err(|e| {
                GooglePlayDeveloperApiKeyInvalid::with_debug(
                    "Google Play API service account authenticator could not be built",
                    &e,
                )
            })?;

        let scopes = &["https://www.googleapis.com/auth/androidpublisher"];
        Ok(authenticator
            .token(scopes)
            .await
            .map_err(|e| {
                GooglePlayDeveloperApiKeyInvalid::with_debug(
                    "Google Play API service account token could not be built",
                    &e,
                )
            })?
            .token()
            .ok_or(GooglePlayDeveloperApiKeyInvalid::new(
                "Google Play API service account token is empty",
            ))?
            .to_string())
    }

    async fn callout<T: DeserializeOwned + 'static>(
        &self,
        url: &str,
        function_name: &str,
        method: Method,
    ) -> Result<T, ServerError> {
        let client = reqwest::Client::new();
        let builder = match method {
            Method::Post => client.post(url),
            Method::Get => client.get(url),
        };
        let response = builder
            .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
            .header(CONTENT_LENGTH, "0")
            .send()
            .await
            .map_err(|e| {
                GooglePlayDeveloperApiError::with_debug(function_name, "callout failed to send", &e)
            })?;

        if !response.status().is_success() {
            return Err(GooglePlayDeveloperApiError::with_debug(
                function_name,
                &format!(
                    "callout returned with {} status code",
                    response.status().to_string(),
                ),
                &response.text().await.unwrap_or_default(),
            ));
        }

        // NOTE:
        //   Response from callout does not contain Authorization header (for
        //   Google, only server-to-server notifications do).

        if TypeId::of::<T>() == TypeId::of::<()>() {
            return Ok(unsafe { std::mem::zeroed() }); // Safe because () has no data.
        }

        response.json().await.map_err(|e| {
            GooglePlayDeveloperApiError::with_debug(
                function_name,
                "failed to parse callout response",
                &e,
            )
        })
    }
}
