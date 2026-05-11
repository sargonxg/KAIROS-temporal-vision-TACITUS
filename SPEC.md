# KAIROS Project Spec

KAIROS is a Rust library and live web demo that gives LLMs temporal vision. It extracts dates, events, actors, commitments, and coherent episodes from prose, then computes Allen-13 temporal relationships across those episodes.

The MVP is intentionally one binary and one Cloud Run service:

- `kairos-core`: temporal primitives, extractors, episode detection, reconciliation, Allen-13 relations, embedded Cozo store.
- `kairos-server`: Axum API and embedded web UI.
- `web/`: TACITUS-flavored timeline interface.
- `deploy.sh`: one-command Cloud Run deployment once `PROJECT_ID` and `GEMINI_API_KEY` are set.

Definition of done:

- `cargo build --release` succeeds.
- `cargo test --release` passes with mock LLM mode.
- `cargo run --release --bin kairos-server` serves `http://localhost:8080`.
- Demo analysis returns at least three episodes and at least one non-before/after Allen relation.
- Cloud Run deployment is handled by `deploy.sh`.
