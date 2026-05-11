# KAIROS Project Spec

KAIROS is a Rust library and live web demo that gives LLMs temporal vision. It extracts dates, events, actors, commitments, and coherent episodes from prose, then computes Allen-13 temporal relationships across those episodes.

KAIROS is intended to become a reusable TACITUS backbone component. It should help policy products see time the way a vision model sees space: not as flat text, but as structured scenes with boundaries, overlapping objects, continuity, and change.

The MVP is intentionally one binary and one Cloud Run service:

- `kairos-core`: temporal primitives, extractors, episode detection, reconciliation, Allen-13 relations, embedded Cozo store.
- `kairos-server`: Axum API and embedded web UI.
- `web/`: TACITUS-flavored timeline interface.
- `deploy.sh`: one-command Cloud Run deployment once `PROJECT_ID` and `GEMINI_API_KEY` are set, or deterministic mock deployment with `KAIROS_LLM=mock`.
- `PUBLIC_LB=true ./deploy.sh`: optional Google Cloud external HTTP load-balancer front door for orgs where direct `run.app` URLs return Google edge 404s.
- Request-scoped Gemini keys: testers can provide `gemini_api_key` and optional `gemini_model` in one `/api/analyze` call or paste them into the UI. KAIROS must not persist them or return them in `AnalysisResult`.

MVP+ API contract:

- `/api/analyze`: stable compatibility endpoint.
- `/api/v1/analyze`: versioned alias for new clients.
- `/api/v1/validate`: accepts extracted graph arrays and returns diagnostics without another LLM call.
- `AnalysisResult.metadata`: schema version, provider, model, mode, input size, elapsed milliseconds.
- `AnalysisResult.diagnostics`: warnings, relation counts, non-trivial relation count, overlap pairs, open-ended episode count, deadline commitment count, unresolved dates, confidence summary.
- Existing arrays remain stable: `dates`, `events`, `actors`, `commitments`, `episodes`, `relations`.

MVP+ UI contract:

- Gemini key/model controls are visible on screen and are request-scoped only.
- The workbench shows timeline, temporal diagnostics, temporal brief, actor lanes, annotated source, ACO extraction, filtered Allen relations, raw JSON, and browser-only analyst corrections.
- Downloaded JSON includes `analyst_corrections`; the server does not persist corrections in this MVP+ pass.

Definition of done:

- `cargo build --release` succeeds.
- `cargo test --release` passes with mock LLM mode.
- `cargo run --release --bin kairos-server` serves `http://localhost:8080`.
- Demo analysis returns at least eight episodes and multiple non-before/after Allen relations in mock mode.
- Smoke script verifies `/healthz`, `/`, `/api/analyze`, `/api/v1/analyze`, `/api/v1/validate`, Gemini controls, and no key leakage.
- Cloud Run deployment is handled by `deploy.sh`.

Backbone constraints:

- Deterministic Rust code owns dates, interval math, relation computation, and API contracts.
- LLMs propose events, ACO primitives, and episode boundaries, but outputs stay typed and reviewable.
- Future learning must come from persisted sessions and human boundary corrections, not hidden state.
- TACITUS products should be able to reuse `kairos-core` without running the demo UI.
