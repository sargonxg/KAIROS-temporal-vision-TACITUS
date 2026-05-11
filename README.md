# KAIROS

**Temporal vision for policy intelligence.** KAIROS is a Rust-first TACITUS tool that turns long-form prose into dates, events, actors, commitments, canonical episodes, and Allen-13 temporal relations.

It is built for the problem normal LLM summaries flatten: policy time is not a list. Sanctions overlap negotiations. Leadership tenures contain crises. Review windows meet implementation phases. Commitments drift, expire, restart, or become institutions. KAIROS gives downstream agents a structured time scene instead of a paragraph blur.

```text
TACITUS // KAIROS
temporal perception -> episode graph -> policy reasoning
```

Live public demo:

```text
http://34.54.231.53
```

The Cloud Run service is deployed in `kairos-temporal-tacitus`. In this Google org, direct `run.app` URLs can return a Google edge 404 even when the revision is healthy, so the current public demo is exposed through a Google Cloud external HTTP load balancer backed by a Cloud Run serverless NEG.

## What Is Live Now

- Rust/Axum service with embedded web workbench.
- Deterministic mock mode for stable public demos and CI.
- Request-scoped Gemini testing from the screen.
- `/api/analyze` and `/api/v1/analyze` temporal extraction endpoints.
- `/api/v1/validate` diagnostics endpoint for already-extracted graphs.
- Metadata and temporal diagnostics on every analysis result.
- Browser-only analyst corrections included in downloaded JSON.

Research-track items such as persistent multi-document memory, PyO3 bindings, WASM graph solving, full TimeML import/export, and cross-document coreference are roadmap items, not yet shipped.

## What It Does

```mermaid
flowchart LR
  A[Policy brief / chronology / memo] --> B[Date extractor]
  A --> C[Event extractor]
  A --> D[ACO actor + commitment extractor]
  B --> E[Episode detector]
  C --> E
  D --> E
  E --> F[Episode reconciler]
  F --> G[Allen-13 relation graph]
  G --> H[Timeline UI + JSON API]
```

KAIROS outputs:

- `dates`: character spans, resolved timestamps, fuzziness.
- `events`: canonical temporal events with anchors.
- `actors`: TACITUS ACO actor primitives.
- `commitments`: who committed what, to whom, in what state.
- `episodes`: coherent intervals with boundaries, anchors, confidence, and review state.
- `relations`: pairwise Allen-13 temporal relations.

## Why It Matters

```mermaid
timeline
  title Meridian Compact Crisis
  2024-01 : Basin scarcity crisis begins
  2024-02 : Compact negotiation opens
  2024-03 : Water-broker sanctions start
  2024-05 : Meridian Compact signed
  2024-06 : Implementation drift appears
  2024-09 : Verification cell emerges
  2024-12 : Reed review phase begins
  2025-02 : Temporal-monitoring unit proposed
```

KAIROS is designed to become a backbone for TACITUS products:

- PRAXIS can reason about crisis windows, implementation slippage, and review phases.
- DIALECTICA can ground arguments in what was true before, during, and after an episode.
- TACITUS analysis agents can query temporal structure instead of re-reading raw text.

## Rust Architecture

```mermaid
flowchart TB
  subgraph core[kairos-core]
    T[temporal.rs<br/>Validity, transaction, bitemporal intervals]
    R[relations.rs<br/>Allen-13 algebra]
    A[aco.rs<br/>Actor, commitment, event primitives]
    E[episode.rs<br/>Episode + boundary types]
    X[extract/*<br/>dates, events, ACO]
    D[detect/*<br/>detector trait + LLM judge]
    P[pipeline.rs<br/>orchestrator]
    S[store.rs<br/>embedded Cozo]
  end
  subgraph server[kairos-server]
    HTTP[Axum API]
    UI[Embedded hacker-style UI]
  end
  P --> HTTP
  UI --> HTTP
```

Core invariants:

- One deployable binary.
- Fast Rust interval and relation logic.
- Embedded static UI.
- Embedded CozoDB, no external database for the MVP.
- Gemini, Ollama, and deterministic mock modes.
- Request-scoped Gemini key testing from the UI.

## Run Locally

```bash
cargo run --release --bin kairos-server
```

Open `http://localhost:8080`, click `Load demo text`, then `Analyze`.

For deterministic local demo/CI:

```bash
export KAIROS_LLM=mock
cargo run --release --bin kairos-server
```

For server-side Gemini:

```bash
export GEMINI_API_KEY="..."
export KAIROS_GEMINI_MODEL="gemini-2.5-flash"
cargo run --release --bin kairos-server
```

You can also paste a Gemini API key into the web UI for a single analysis run. The key is sent only in that `/api/analyze` request, is not saved by the browser, is not stored by the server, and is not included in exported JSON.

## API

```bash
curl -s -X POST http://localhost:8080/api/analyze \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "January 15, 2024: Riverdale announces rationing. March 12, 2024: leadership changes.",
    "gemini_api_key": "optional-per-request-key",
    "gemini_model": "gemini-2.5-flash"
  }'
```

Response shape:

```json
{
  "session_id": "sess_...",
  "metadata": {
    "schema_version": "kairos.analysis.v1",
    "provider": "mock",
    "model": "deterministic-demo",
    "mode": "deterministic_mock",
    "input_chars": 2840,
    "elapsed_ms": 12
  },
  "diagnostics": {
    "warnings": [],
    "relation_counts": {"Before": 14, "Overlaps": 6},
    "non_trivial_relations": 42,
    "open_ended_episodes": 0,
    "dense_overlap_pairs": [],
    "deadline_commitments": 4,
    "unresolved_dates": 0,
    "confidence": {"episode_min": 0.9, "episode_avg": 0.91, "episode_max": 0.92}
  },
  "dates": [],
  "events": [],
  "actors": [],
  "commitments": [],
  "episodes": [],
  "relations": []
}
```

`gemini_api_key` and `gemini_model` are optional and request-scoped. When a key is present, that analysis uses Gemini even if the deployed server default is mock mode.

Validate an existing graph without another LLM call:

```bash
curl -s -X POST http://localhost:8080/api/v1/validate \
  -H 'Content-Type: application/json' \
  -d '{"dates":[],"commitments":[],"episodes":[],"relations":[]}'
```

## Demo Gate

The built-in Meridian Compact Crisis mock demo currently returns:

```text
23 dates
17 events
7 actors
6 commitments
8 episodes
56 Allen relations
42 non-trivial relations
```

That is the minimum public demo posture: it should visibly prove overlapping temporal structure, not just chronology extraction.

## Deploy To Cloud Run

```bash
export PROJECT_ID="your-project-id"
export GEMINI_API_KEY="..."
./deploy.sh
```

For a no-key deterministic Cloud Run demo:

```bash
PROJECT_ID="your-project-id" KAIROS_LLM=mock ./deploy.sh
```

If your org blocks or disables direct `run.app` URLs, create the public load-balancer front door too:

```bash
PROJECT_ID="your-project-id" KAIROS_LLM=mock PUBLIC_LB=true ./deploy.sh
```

The deploy script enables required APIs, creates Artifact Registry if needed, stores the Gemini key in Secret Manager for Gemini mode, builds with Cloud Build, deploys Cloud Run, and then disables the Cloud Run Invoker IAM check for public testing when permitted.

## Verification

```bash
cargo fmt --all --check
cargo test --release
cargo build --release
powershell -ExecutionPolicy Bypass -File scripts/kairos-smoke.ps1 -BaseUrl http://localhost:8080
```

Read more:

- `docs/ARCHITECTURE.md` explains the Rust crates, temporal model, Allen-13 relation layer, and learning path.
- `docs/CAPABILITY_MAP.md` tracks what is live now and what turns KAIROS into a TACITUS backbone.
- `docs/ROADMAP_MVP_PLUS.md` converts the deep research brief into an implementation backlog.
- `examples/demo-text.md` contains the Meridian Compact Crisis.

## License

Apache-2.0. Copyright 2026 TACITUS.
