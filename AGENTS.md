# fractic-iap

Stateless in-app-purchase verification for Apple (App Store Server API and
server notifications, JWS) and Google (Play Developer API, RTDN
notifications): `IapUtil` returns domain `IapDetails` /
`IapUpdateNotification`, vendor structs never leave the crate.
`IapCtxView` carries the credentials. Public, provided as-is
(`fractic-io/rust-iap`, identity `fractic`). Consumed by
`app/backend` (`purchase` resource).

## Read first

- `~/.agents/AGENTS.md` (workspace orientation; loaded automatically by Claude
  Code, read it if you are Codex or Grok) and `~/.agents/guides/rust.md`.
- `_/codebase.md`: structure, execution flow, visibility strategy, how to
  add a platform.
- `_/domain_knowledge.md`: Apple vs Google vocabulary, purchase states,
  sandbox detection, prices, security notes.

## Layout

Library with `src/impl/mod.rs` as the gatekeeper (`impl/domain/{entities,
repositories}`, `impl/data/{datasources,models/<vendor api>,repositories}`,
`impl/exports`), plus `constants.rs`, `context.rs`, `errors.rs`, `util.rs`
(the only public touchpoint). `res/trust/` holds Apple root and WWDR
certificates embedded for JWS verification.

## Working here

- No flake here: `nix develop /efs/fractic/storyengine/util --command cargo
  test`; keep `cargo clippy --all-targets --all-features -- -D warnings`
  clean.
- Work one bounded task at a time; remove temporary diagnostics before
  committing. If a high-impact refactor becomes obvious, describe it in the
  PR (or a `_/refactors.md`) instead of doing it in passing.
- New platform support follows `_/codebase.md` §5: datasource module, domain
  entity extensions, thin adapter in `IapRepositoryImpl`; `IapUtil` stays
  stable.
- Security-sensitive: verify signatures with the embedded trust roots and the
  Google JWK set; never log tokens or receipts; secrets are never written to
  disk.
- When a change alters structure or behaviour, update `_/codebase.md` /
  `_/domain_knowledge.md` in the same PR.
- Commit / PR titles: imperative, trailing period.
