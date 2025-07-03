# Domain Knowledge

This section captures tacit knowledge that shortens the feedback loop for agents working on the crate. Use it as a quick mental model, not as exhaustive documentation.

---

## Apple vs Google vocabulary
• Apple *transactionId* ≈ Google *purchaseToken*.
• Apple *productId* / *bundleId* ⇢ Google *sku*.
• Apple transmits server notifications as **JWS**; Google uses **JWT** plus an RTDN header signature.

## Purchase state model
• *Active* ⇒ entitlement should be granted.
• Subscriptions in grace period are considered *inactive*.
• Apple marks consumables as consumed immediately; Google requires an explicit `consume` call.

## Sandbox vs Production
• Apple sets `is_sandbox` inside the JWS payload.
• Google infers sandbox from the package name suffix (`.debug`, `.dev`).

## Price information
• Prices are micro-units (1 000 000 µ = 1 unit). Treat them as `i64`.
• Currency codes are ISO-4217; some codes are partner-specific – validate.

## Security notes
• Validate Google RTDN against the global JWK set (`GOOGLE_JWK_URL`), rotating approximately every few hours.
• Apple's JWS signature requires an OAuth 2.0 bearer token with scope `appstoreconnect-v1`.

## Implicit guarantees
• Secrets are never written to disk.
• All network operations timeout after 5 s; caller decides on retries.