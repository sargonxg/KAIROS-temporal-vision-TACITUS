# Local native checkpoint · 12 November 2026

Rust 1.95.0 locked workspace check, formatting, Clippy with warnings denied and 9 tests pass locally. Uses derived defaults with the same explicit enum default variants, pins Rust to 1.95.0 and adds read-only native CI. No temporal interpretation, inference, API contract or source data is changed.

The exact accepted source is recorded by the accompanying PR and CI. Run `cargo fmt --all -- --check`, `cargo check --locked --workspace --all-targets`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, then `cargo test --locked --workspace`. Existing fixtures/tests cover the changed code; these checks do not establish live provider, model-quality, hosted authentication or analytical benefit.

This remains bounded research with a distinct contract. It is not promoted to a dependency of the current invited App. No deployment, paid service, automatic extraction or cross-application format conversion is enabled.
