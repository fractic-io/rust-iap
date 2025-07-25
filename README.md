# fractic-iap

Stateless helper for:

* Verifying Apple App Store & Google Play purchases (consumable, non-consumable, subscription).
* Parsing server-to-server update notifications (refunds, cancellations, expiry changes, …).

The crate is transport-agnostic and does not persist anything – bring your own database / message bus.

---

## Option 1:

Use [fractic-context](https://github.com/fractic-io/rust-context) to load credentials once at startup and share them application-wide.

```rust
use fractic_context::define_ctx;
use fractic_iap::{util::IapUtil, IapCtxView};
use fractic_server_error::ServerError;

// ▶ 1. Declare the application-level context.
//    Secrets are fetched from AWS Secrets Manager secret `SECRETS_ID`
//    located in region `SECRETS_REGION` (env vars).
//    The secret JSON must contain the keys shown below.
define_ctx! {
    name: Ctx,
    env {},
    secrets_fetch_region: SECRETS_REGION,
    secrets_fetch_id: SECRETS_ID,
    secrets {
        GOOGLE_API_KEY: String,
        APPLE_API_KEY: String,
        APPLE_KEY_ID:  String,
        APPLE_ISSUER_ID: String,
    },
    deps {},
    // Expose only what fractic-iap needs.
    views { IapCtxView }
}

async fn run() -> Result<(), ServerError> {
    // ▶ 2. Initialise context once.
    let ctx = Ctx::init().await?;

    // ▶ 3. Construct the utility.
    let iap = IapUtil::from_context(
        &*ctx,
        "com.example.app",        // application / bundle id
        "expected_aud_claim",    // JWT `aud` claim to require
    ).await?;

    // Verify a purchase originating from Apple.
    use fractic_iap::domain::{
        entities::{iap_product_id::IapNonConsumableId, iap_purchase_id::IapPurchaseId},
        entities::iap_details::{IapDetails, NonConsumableDetails},
    };

    let details: IapDetails<NonConsumableDetails> = iap
        .verify_and_get_details(
            IapNonConsumableId("product_sku".into()),
            IapPurchaseId::AppStoreTransactionId("tx_id".into()),
            /* include_price */ true,
        )
        .await?;

    // … handle `details` / notifications etc.
    Ok(())
}
```

---

## Option 2:

If you prefer to inject secrets manually:

```rust
use fractic_iap::util::IapUtil;

let iap = IapUtil::from_values(
    "com.example.app",           // application / bundle id
    "expected_aud_claim",        // JWT `aud` claim to require
    "<apple_api_key>",
    "<apple_key_id>",
    "<apple_issuer_id>",
    "<google_api_key>",
).await?;
```

---

### Feature status

* ✅ Purchase verification for Apple / Google (all product types).
* ✅ Server-to-server notifications parsing (most events).
* ⬜ Subscription tier change events (Apple only).
* ⬜ Subscription price change events.