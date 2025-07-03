# Agent Guidelines

1. **Focus** – Work on one bounded task at a time; resist bundling.

2. **Quality Gate** – Ensure
   ```bash
   cargo check
   cargo clippy --all-targets --all-features -D warnings
   ```
   are clean before submitting work.

3. **Ephemeral Scaffolding** – Remove temporary diagnostics, comments or code that will be obsolete in three months.

4. **Stewardship** – After finishing your task, consider long-term maintainability. If a high-impact refactor is clear, document it in `_refactors.md`; do not implement it immediately.