# Stack

Language: Rust 2021 (≥1.76).

Key crates
• `reqwest` (TLS: rustls) – HTTP client.
• `jsonwebtoken`, `jwtk` – JWS/JWT.
• `yup-oauth2` – Apple OAuth.
• `async-trait` – async in traits.
• `fractic-env-config` – secret orchestration.
• `fractic-server-error` – ergonomic error handling.

No binary targets, only a library crate. All async APIs are executor-agnostic.