# fractic-iap

Stateless in-app-purchase verification for Apple (App Store Server API and
server notifications, JWS) and Google (Play Developer API, RTDN
notifications). `IapUtil` is the only public touchpoint and returns domain
`IapDetails` / `IapUpdateNotification`; vendor structs never leave the
crate. `IapCtxView` carries the credentials. Consumed by `app/backend`
(`purchase` resource). Library layout per `/efs/fractic/_/rust.md` §2:
`src/impl/mod.rs` gates `impl/domain/{entities,repositories}`,
`impl/data/{datasources,models/<vendor api>,repositories}` and
`impl/exports`; plus `constants.rs`, `context.rs`, `errors.rs`, `util.rs`.
`res/trust/` holds the Apple root and WWDR certificates embedded for JWS
verification.

Flow: `IapUtil::from_context` / `from_values` builds an `IapRepositoryImpl`
wiring the Apple and Google datasources; `verify_and_get_details` →
`IapRepository` → datasource (`app_store_server_api` /
`google_play_developer_api`) → raw JSON / JWS models → `IapDetails<T>`;
`parse_*_notification` validates the signature (Google JWK set, Apple
OAuth → JWS) → `IapUpdateNotification`; `consume` delegates to Google Play,
a no-op for Apple by contract.

- Keep `cargo clippy --all-targets --all-features -- -D warnings` clean.
- A new platform is a datasource module implementing the HTTP contract,
  extensions to `TypedProductId` and the domain entities if new concepts
  arise, and a thin adapter in `IapRepositoryImpl`; `IapUtil` stays stable.
- Security-sensitive: verify signatures with the embedded trust roots and
  the Google JWK set; never log tokens or receipts; secrets are never
  written to disk. Purchase history is the caller's business, not this
  crate's.
- Commit / PR titles: imperative, trailing period.

## Docs

- `_/domain_knowledge.md`: Apple vs Google vocabulary, purchase states,
  sandbox detection, prices, security notes.
