Rust utility for validating in-app purchases (IAP) made through the Apple App Store or Google Play Store, and parsing server-to-server change notifications (for refunds, subscription cancellations, etc.).

It should be able to handle all purchase types (Consumable, NonConsumable, Subscription), and almost all server notification types. It currently does not support differentiating between different tiers of a given subscription (tier upgrade/downgrade notifications are ignored), since that's an Apple-only feature. Subscription price change notifications are also ignored, but could be added fairly easily.

This code is provided as-is. For the time being, attention will not be given to backwards compatibility or clear documentation. It is open-sourced mainly for the chance that snippets may be useful to others looking to do similar tasks. Eventually, this may become a real library, well-documented, etc., for uploading to crates.io.

## Usage

### Option 1: Using AWS Secrets Manager

To use API keys stored in AWS Secrets Manager:

```rust
use fractic_aws_secrets::{
    define_secrets_config,
    env::{SECRETS_ID, SECRETS_REGION},
    load_secrets, SecretsConfigEnum,
};
use fractic_env_config::{define_env_config, load_env, EnvConfigEnum};
use fractic_generic_server_error::GenericServerError;
use fractic_iap::{
    domain::entities::{
        iap_details::{IapDetails, NonConsumableDetails, SubscriptionDetails},
        iap_product_id::{IapNonConsumableId, IapSubscriptionId},
        iap_purchase_id::IapPurchaseId,
        iap_update_notification::{IapUpdateNotification, NotificationDetails},
    },
    secrets::{APPLE_API_KEY, APPLE_ISSUER_ID, APPLE_KEY_ID, GOOGLE_API_KEY},
    util::IapUtil,
};

// The set-up below causes the fractic-aws-secrets library to fetch secret ID
// "SECRETS_ID" from region "SECRETS_REGION", and expects it to be structured as
// JSON with the keys "GOOGLE_API_KEY", "APPLE_API_KEY", ...
// ---------------------------------------------------------

// Set up environment variables.
//
// This config causes the fractic-env-config library to fetch the environment
// variables "SECRETS_REGION" and "SECRETS_ID", which are the minimum keys
// required for the config to be accepted by fractic-aws-secrets.
define_env_config!(
    EnvConfig,
    SecretsRegion => SECRETS_REGION,
    SecretsId => SECRETS_ID,
);

// Set up secrets.
//
// This config causes the fractic-aws-secrets library to fetch the keys
// "GOOGLE_API_KEY", "APPLE_API_KEY", ..., from the AWS Secrets Manager secret
// JSON at "SECRETS_ID", which are the minimum keys required for the config to
// be accepted by the IAP util.
define_secrets_config!(
    SecretsConfig,
    GoogleApiKey => GOOGLE_API_KEY,
    AppleApiKey => APPLE_API_KEY,
    AppleKeyId => APPLE_KEY_ID,
    AppleIssuerId => APPLE_ISSUER_ID,
);

async fn run() -> Result<(), GenericServerError> {
    // Load the env / secrets.
    let env = load_env::<EnvConfig>()?;
    let secrets = load_secrets::<SecretsConfig>(env.clone_into()?).await?;

    // Load the utility.
    let iap_util = IapUtil::from_secrets(
        secrets.clone_into()?,
        // Application ID (also referred to as bundle ID or package name).
        "com.example.appid",
        // When validating API payloads, only tokens with matching 'aud' claim
        // will be accepted.
        "<expected_aud_claim>"
    ).await?;

    // Verify a purchase from Apple.
    let apple_purchase: IapDetails<NonConsumableDetails> = iap_util
        .verify_and_get_details(
            IapNonConsumableId("product_sku".into()),
            IapPurchaseId::AppStoreTransactionId("transaction_id".into()),
            /* include_price_info: */ true,
            /* error_if_not_active: */ true,
        )
        .await?;

    // Verify a purchase from Google.
    let google_purchase: IapDetails<SubscriptionDetails> = iap_util
        .verify_and_get_details(
            IapSubscriptionId("product_sku".into()),
            IapPurchaseId::GooglePlayPurchaseToken("token".into()),
            /* include_price_info: */ true,
            /* error_if_not_active: */ true,
        )
        .await?;

    // Parse a notification from Apple.
    let body: &str = "...";
    let notification: IapUpdateNotification = iap_util
        .parse_apple_notification(body)
        .await?;
    match notification.details {
        NotificationDetails::Test => {}
        NotificationDetails::ConsumableVoided { .. } => { /* handler */ }
        NotificationDetails::NonConsumableVoided { .. } => { /* handler */ }
        NotificationDetails::SubscriptionEnded { .. } => { /* handler */ }
        NotificationDetails::SubscriptionExpiryChanged { .. } => { /* handler */ }
        /* etc. */
    }

    // Parse a notification from Google.
    let authorization: &str = "..."; // Request's "Authorization" header.
    let body: &str = "...";
    let google_notification: IapUpdateNotification = iap_util
        .parse_google_notification(authorization, body)
        .await?;
    match notification.details {
        ...
    }

    Ok(())
}
```

### Option 2: Hardcoding Secrets

To inline the API keys directly:

```rust
use fractic_generic_server_error::GenericServerError;
use fractic_iap::util::IapUtil;

async fn run() -> Result<(), GenericServerError> {
    let iap_util = IapUtil::from_values(
        "com.example.appid",
        "<expected_aud_claim>",
        "Apple API Key",
        "Apple Key ID",
        "Apple Issuer ID",
        "Google API Key",
    ).await?;

    let purchase = iap_repository
        .verify_and_get_details(
            ...
        )
        .await?;

    ...
}
```