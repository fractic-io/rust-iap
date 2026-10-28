# Domain knowledge

## Apple vs Google vocabulary

- Apple `transactionId` ≈ Google `purchaseToken`.
- Apple `productId` / `bundleId` ≈ Google `sku`.
- Apple sends server notifications as JWS; Google uses JWT plus an RTDN
  header signature.

## Purchase state model

- *Active* means the entitlement should be granted.
- A subscription in its grace period counts as *inactive*.
- Apple marks consumables as consumed immediately; Google requires an
  explicit `consume` call.

## Sandbox vs production

- Apple sets `is_sandbox` inside the JWS payload.
- Google sandbox is inferred from the package name suffix (`.debug`, `.dev`).

## Prices

- Prices are micro-units (1 000 000 µ = 1 unit), handled as `i64`.
- Currency codes are ISO-4217, but some codes are partner-specific: validate.

## Security

- Google RTDN is validated against the global JWK set (`GOOGLE_JWK_URL`),
  which rotates every few hours.
- Apple's JWS verification needs an OAuth 2.0 bearer token with scope
  `appstoreconnect-v1`.
- All network operations time out after 5 s; the caller decides on retries.
