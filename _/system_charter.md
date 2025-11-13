# System Charter

The Fractic IAP crate is the canonical source of truth for **"does the customer actually own this product?"** across Apple and Google application stores.

Motivating outcomes
• **Server-side certainty** Purchase verification and notification parsing are cryptographically sound and aligned with each vendor's evolving contracts.
• **Unified domain** Both vendor taxonomies are projected onto a single, minimal set of business concepts (`IapDetails`, `IapUpdateNotification`, etc.) so downstream services never deal with platform quirks.
• **Operational focus** Product code calls a single façade (`IapUtil`) without having to track vendor API changes, OAuth mechanics or JWS nuances.
• **Long horizon** The design favours clear boundaries, deterministic behaviour and easy refactoring over momentary optimisation.

Non-goals
• Client-side SDK duplication.
• Storing purchase history (that belongs in the caller).
• Handling third-party payment processors.