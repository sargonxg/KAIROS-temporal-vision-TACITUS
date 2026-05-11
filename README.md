# KAIROS

**Temporal vision for LLMs.** KAIROS reads prose, extracts dates and events, detects coherent episodes, and computes Allen-13 temporal relationships between them. It is built for TACITUS workflows where sequence, overlap, tenure, commitments, and institutional change matter.

KAIROS ships as one Rust 1.95 binary: Axum server, embedded web UI, in-process CozoDB, and a pluggable LLM adapter. Gemini is the default production model, Ollama is supported for local open-source mode, and `KAIROS_LLM=mock` gives deterministic CI/demo output.

## Run Locally

```bash
cargo run --release --bin kairos-server
```

Open `http://localhost:8080`, click `Load demo text`, then `Analyze`.

For real model extraction:

```bash
export GEMINI_API_KEY="..."
cargo run --release --bin kairos-server
```

For deterministic local demo/CI:

```bash
export KAIROS_LLM=mock
cargo run --release --bin kairos-server
```

## API

```bash
curl -s -X POST http://localhost:8080/api/analyze \
  -H 'Content-Type: application/json' \
  -d '{"text":"January 15, 2024: Riverdale announces rationing. March 12, 2024: leadership changes."}'
```

Response shape:

```json
{
  "session_id": "sess_...",
  "dates": [],
  "events": [],
  "actors": [],
  "commitments": [],
  "episodes": [],
  "relations": []
}
```

## Deploy To Cloud Run

Create or select a Google Cloud project with billing enabled, then:

```bash
export PROJECT_ID="your-project-id"
export GEMINI_API_KEY="..."
./deploy.sh
```

The script enables required APIs, creates Artifact Registry if needed, stores the Gemini key in Secret Manager, builds with Cloud Build, and deploys a public Cloud Run service.

For a no-key deterministic Cloud Run demo:

```bash
PROJECT_ID="your-project-id" KAIROS_LLM=mock ./deploy.sh
```

Some TACITUS organization policies may block public `allUsers` invoker bindings. In that case, use an authenticated request with `gcloud auth print-identity-token`.

## Architecture

```text
TEXT -> regex dates -> LLM events -> LLM ACO actors/commitments
     -> episode detector -> reconciler -> Allen-13 relations
     -> JSON + interactive timeline UI
```

Core invariants:

- One deployable binary.
- Embedded static UI.
- Embedded CozoDB, no external database.
- Gemini/Ollama/mock LLM modes selected by environment.
- Apache-2.0 open-source repo posture.

## Acceptance

```bash
cargo fmt --check
cargo test --release
cargo build --release
```

Optional Cloud Build deploy:

```bash
PROJECT_ID="your-project-id" GEMINI_API_KEY="..." ./deploy.sh
```

## License

Apache-2.0. Copyright 2026 TACITUS.
