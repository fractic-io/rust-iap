# Codebase

## 1. Structure

```
src/
│
├── impl/              ← internal implementation (not exported)
│   ├── domain/        ← entities + repository traits
│   │   ├── entities/
│   │   │   └── *.rs
│   │   └── repositories/
│   │       └── iap_repository.rs
│   │
│   └── data/          ← implementation details
│       ├── datasources/   (HTTP calls, cryptography, helpers)
│       ├── models/        (serde structs for external JSON/JWS)
│       └── repositories/  (IapRepositoryImpl: orchestrates datasources)
│
├── util.rs            ← façade, only public touchpoint for end-users
├── constants.rs
├── errors.rs
├── secrets.rs
└── lib.rs             ← re-exports + public API surface
```

`src/impl/mod.rs` is the *single* gatekeeper. Everything under `impl::domain` and `impl::data` is declared `pub(crate)`; the outside world sees only what `impl::exports` re-exports and what `lib.rs` further surfaces.

```
extern crate fractic_iap;

use fractic_iap::{
    entities::{IapDetails, IapPurchaseId, IapUpdateNotification, …},
    util::IapUtil,
};
```

## 2. Execution Flow

1. **Construction** – `IapUtil::from_secrets` / `from_values` produces an `IapRepositoryImpl`, wiring Apple & Google datasources with the caller's credentials.
2. **Verification** – `IapUtil::verify_and_get_details` → domain trait `IapRepository` → `IapRepositoryImpl` → datasource (`app_store_server_api` or `google_play_developer_api`) → raw JSON/JWS models → domain `IapDetails<T>`.
3. **Notification parsing** – `IapUtil::parse_*_notification` validates signature (Google JWK or Apple OAuth → JWS) → domain `IapUpdateNotification`.
4. **Consumption** – `IapUtil::consume` delegates to Google Play API; Apple is a no-op by contract.

All outward-facing functions return *domain* types; vendor-specific structs never leave the crate.

## 3. Visibility Strategy

• `pub(crate)` everywhere by default – only deliberate exports escape.  
• `impl::exports` aggregates the public subset so external callers import from a single cohesive namespace.  
• Internal helpers (`data::datasources::utils`, `constants.rs`, etc.) remain invisible; this guarantees we can refactor without semver bumps.

## 4. The Four Causes

• **Material** – Concrete Rust structs and enums: `IapDetails`, `IapUpdateNotification`, external JSON models.  
• **Formal** – Traits and invariants: `IapRepository`, `TypedProductId`, state machine encoded in enums.  
• **Efficient** – Datasource modules perform HTTP calls, signature checks and JSON parsing; `IapRepositoryImpl` orchestrates them.  
• **Final** – Provide *business certainty* about purchase ownership, abstracted away from vendor mechanics.

## 5. Extending the Crate

New platform support should follow the same pattern:
1. Add *datasource* module implementing the HTTP contract.
2. Extend `TypedProductId` & domain entities if new concepts arise.
3. Implement a thin adapter inside `IapRepositoryImpl`.

No extra work is needed at the API layer; `IapUtil` remains stable.