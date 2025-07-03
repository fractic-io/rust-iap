# Potential Refactors

## 1. Decompose `iap_repository_impl.rs`

**Problem** – ≈800 lines interleave orchestration, network I/O and parsing, making the module hard to reason about and to unit-test.

**Proposal**
1. Move *verification*, *notification parsing* and *consumption* logic into dedicated modules under `src/impl/data/repositories/`.
2. Keep `iap_repository_impl.rs` as a thin coordinator that implements `IapRepository` by delegating to the new modules.
3. Mark the new modules `pub(crate)`; only the trait implementation is of interest to the rest of the crate.

**Impact**
• Sharper separation of concerns and lower cognitive load.  
• Simplified mocking and isolated testing.  
• Reduced merge-conflict surface when multiple agents work in parallel.  
• **No** change to the public API.