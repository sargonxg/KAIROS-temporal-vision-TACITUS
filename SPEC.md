# KAIROS Project Spec

KAIROS is a Rust library and live web demo that gives LLMs temporal vision. It extracts dates, events, actors, commitments, and coherent episodes from prose, then computes Allen-13 temporal relationships across those episodes.

KAIROS is intended to become a reusable TACITUS backbone component. It should help policy products see time the way a vision model sees space: not as flat text, but as structured scenes with boundaries, overlapping objects, continuity, and change.

The MVP is intentionally one binary and one Cloud Run service:

- `kairos-core`: temporal primitives, extractors, episode detection, reconciliation, Allen-13 relations, embedded Cozo store.
- `kairos-server`: Axum API and embedded web UI.
- `web/`: TACITUS-flavored timeline interface.
- `deploy.sh`: one-command Cloud Run deployment once `PROJECT_ID` and `GEMINI_API_KEY` are set, or deterministic mock deployment with `KAIROS_LLM=mock`.
- Request-scoped Gemini keys: testers can provide `gemini_api_key` in one `/api/analyze` call or paste it into the UI. KAIROS must not persist it or return it in `AnalysisResult`.

Definition of done:

- `cargo build --release` succeeds.
- `cargo test --release` passes with mock LLM mode.
- `cargo run --release --bin kairos-server` serves `http://localhost:8080`.
- Demo analysis returns at least eight episodes and multiple non-before/after Allen relations in mock mode.
- Cloud Run deployment is handled by `deploy.sh`.

Backbone constraints:

- Deterministic Rust code owns dates, interval math, relation computation, and API contracts.
- LLMs propose events, ACO primitives, and episode boundaries, but outputs stay typed and reviewable.
- Future learning must come from persisted sessions and human boundary corrections, not hidden state.
- TACITUS products should be able to reuse `kairos-core` without running the demo UI.
