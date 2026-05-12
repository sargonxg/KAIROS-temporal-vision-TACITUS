# Contributing To KAIROS

KAIROS is TACITUS infrastructure, so contributions should improve trust, evidence quality, temporal reasoning, or developer usability. Avoid changes that make the demo look smarter while weakening source grounding.

## Development Setup

```bash
cargo fmt --all --check
cargo test --release
cargo build --release
node --check web/app.js
```

For the browser smoke test, start the server first:

```bash
KAIROS_LLM=mock cargo run --release --bin kairos-server
powershell -ExecutionPolicy Bypass -File scripts/kairos-smoke.ps1 -BaseUrl http://localhost:8080
```

## Contribution Principles

- Keep `kairos-core` reusable as a Rust library.
- Keep `kairos-server` a thin deployable wrapper.
- Preserve deterministic mock mode for CI and public demos.
- Make LLM-backed behavior provider-agnostic where possible.
- Every extracted or inferred object should carry source evidence when feasible.
- Hypotheses must cite support and must not overwrite extracted facts.
- Do not log or persist request-scoped API keys.
- Update README, docs, and fixtures when behavior changes.

## Pull Request Checklist

- Explain the user-facing purpose of the change.
- Identify whether the change affects mock mode, Gemini mode, or both.
- Add or update tests for new output fields, detectors, or graph edges.
- Run the verification commands above.
- Note any provider-specific behavior or known limitations.

## Project Areas

- `crates/kairos-core/src/pre_read`: deterministic document pre-reading.
- `crates/kairos-core/src/actors`: actor canonicalization and aliases.
- `crates/kairos-core/src/friction.rs`: human friction model.
- `crates/kairos-core/src/graph.rs`: temporal knowledge graph export.
- `crates/kairos-core/src/pipeline.rs`: orchestration contract.
- `web/`: simple public workbench.
- `examples/eval/`: focused fixtures for regression testing.
