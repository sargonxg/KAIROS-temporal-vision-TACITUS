# KAIROS Build Playbook

This repository was scaffolded from the TACITUS KAIROS MVP brief.

## Local Verification

```bash
cargo fmt --check
cargo test --release
cargo build --release
```

## Mock Demo

```bash
export KAIROS_LLM=mock
cargo run --release --bin kairos-server
```

Open `http://localhost:8080`, load the demo text, and analyze. Mock mode returns deterministic Meridian Compact episodes so the UI and relations can be reviewed without a Gemini key.

Expected mock output:

- at least 15 dates;
- at least 15 events;
- at least 7 actors;
- at least 6 commitments;
- at least 8 episodes;
- multiple non-trivial Allen-13 relations.

## Gemini Demo

```bash
export GEMINI_API_KEY="..."
cargo run --release --bin kairos-server
```

## Cloud Run

```bash
export PROJECT_ID="your-project-id"
export GEMINI_API_KEY="..."
./deploy.sh
```

The deploy script enables Cloud Run, Artifact Registry, Cloud Build, and Secret Manager APIs; creates the `kairos` Artifact Registry repository; stores the Gemini key in Secret Manager; builds the container remotely; and deploys the `kairos` service.

For a deterministic deployed demo without a Gemini key:

```bash
PROJECT_ID="your-project-id" KAIROS_LLM=mock ./deploy.sh
```

The current TACITUS org policy may block `allUsers` public invoker bindings. If that happens, the service is still usable with an authenticated identity token.
